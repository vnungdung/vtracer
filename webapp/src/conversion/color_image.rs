use wasm_bindgen::prelude::*;
use visioncortex::{Color, ColorImage, PathSimplifyMode};
use visioncortex::color_clusters::{Clusters, Runner, RunnerConfig, HIERARCHICAL_MAX, IncrementalBuilder, KeyingAction};

use crate::{canvas::*, svg::*};
use serde::Deserialize;
use super::util;

const KEYING_THRESHOLD: f32 = 0.2;

#[derive(Debug, Deserialize, Clone)]
pub struct ColorImageConverterParams {
    pub canvas_id: String,
    pub svg_id: String,
    pub mode: String,
    pub hierarchical: String,
    pub corner_threshold: f64,
    pub length_threshold: f64,
    pub max_iterations: usize,
    pub splice_threshold: f64,
    pub filter_speckle: usize,
    pub color_precision: i32,
    pub layer_difference: i32,
    pub path_precision: u32,
}

#[wasm_bindgen]
pub struct ColorImageConverter {
    canvas: Option<Canvas>,
    svg: Option<Svg>,
    stage: Stage,
    counter: usize,
    mode: PathSimplifyMode,
    params: ColorImageConverterParams,
}

pub enum Stage {
    New,
    Clustering(IncrementalBuilder),
    Reclustering(IncrementalBuilder),
    Vectorize(Clusters),
}

impl ColorImageConverter {
    pub fn new(params: ColorImageConverterParams) -> Self {
        let canvas = Canvas::new_from_id(&params.canvas_id);
        let svg = Svg::new_from_id(&params.svg_id);
        Self {
            canvas: Some(canvas),
            svg: Some(svg),
            stage: Stage::New,
            counter: 0,
            mode: util::path_simplify_mode(&params.mode),
            params,
        }
    }

    pub fn from_bytes(image_data: &[u8], width: usize, height: usize, params: ColorImageConverterParams) -> Self {
        let mut image = ColorImage {
            pixels: image_data.to_vec(),
            width,
            height,
        };

        let key_color = if Self::should_key_image(&image) {
            if let Ok(key) = Self::find_unused_color_in_image(&image) {
                for y in 0..height {
                    for x in 0..width {
                        if image.get_pixel(x, y).a == 0 {
                            image.set_pixel(x, y, &key);
                        }
                    }
                }
                key
            } else {
                Color::default()
            }
        } else {
            Color::default()
        };

        let runner = Runner::new(RunnerConfig {
            diagonal: params.layer_difference == 0,
            hierarchical: HIERARCHICAL_MAX,
            batch_size: 25600,
            good_min_area: params.filter_speckle,
            good_max_area: width * height,
            is_same_color_a: params.color_precision,
            is_same_color_b: 1,
            deepen_diff: params.layer_difference,
            hollow_neighbours: 1,
            key_color,
            keying_action: if params.hierarchical == "cutout" {
                KeyingAction::Keep
            } else {
                KeyingAction::Discard
            },
        }, image);

        Self {
            canvas: None,
            svg: Some(Svg::default()),
            stage: Stage::Clustering(runner.start()),
            counter: 0,
            mode: util::path_simplify_mode(&params.mode),
            params,
        }
    }
}

#[wasm_bindgen]
impl ColorImageConverter {
    pub fn new_with_string(params: String) -> Self {
        let params: ColorImageConverterParams = serde_json::from_str(&params).unwrap();
        Self::new(params)
    }

    pub fn init(&mut self) {
        if let Some(canvas) = &self.canvas {
            let width = canvas.width() as u32;
            let height = canvas.height() as u32;
            let mut image = canvas.get_image_data_as_color_image(0, 0, width, height);

            let key_color = if Self::should_key_image(&image) {
                if let Ok(key) = Self::find_unused_color_in_image(&image) {
                    for y in 0..height as usize {
                        for x in 0..width as usize {
                            if image.get_pixel(x, y).a == 0 {
                                image.set_pixel(x, y, &key);
                            }
                        }
                    }
                    key
                } else {
                    Color::default()
                }
            } else {
                Color::default()
            };

            let runner = Runner::new(RunnerConfig {
                diagonal: self.params.layer_difference == 0,
                hierarchical: HIERARCHICAL_MAX,
                batch_size: 25600,
                good_min_area: self.params.filter_speckle,
                good_max_area: (width * height) as usize,
                is_same_color_a: self.params.color_precision,
                is_same_color_b: 1,
                deepen_diff: self.params.layer_difference,
                hollow_neighbours: 1,
                key_color,
                keying_action: if self.params.hierarchical == "cutout" {
                    KeyingAction::Keep
                } else {
                    KeyingAction::Discard
                },
            }, image);

            self.stage = Stage::Clustering(runner.start());
        }
    }

    pub fn tick(&mut self) -> bool {
        match &mut self.stage {
            Stage::New => panic!("Uninitialized"),
            Stage::Clustering(builder) => {
                if let Some(canvas) = &self.canvas {
                    canvas.log("Clustering tick");
                }
                if builder.tick() {
                    let result = builder.result();
                    if self.params.hierarchical == "cutout" {
                        let view = result.view();
                        let image = view.to_color_image();
                        let runner = Runner::new(RunnerConfig {
                            diagonal: false,
                            hierarchical: 64,
                            batch_size: 25600,
                            good_min_area: 0,
                            good_max_area: image.width * image.height,
                            is_same_color_a: 0,
                            is_same_color_b: 1,
                            deepen_diff: 0,
                            hollow_neighbours: 0,
                            key_color: Color::default(),
                            keying_action: KeyingAction::Discard,
                        }, image);
                        self.stage = Stage::Reclustering(runner.start());
                    } else {
                        self.stage = Stage::Vectorize(result);
                    }
                }
                false
            },
            Stage::Reclustering(builder) => {
                if let Some(canvas) = &self.canvas {
                    canvas.log("Reclustering tick");
                }
                if builder.tick() {
                    self.stage = Stage::Vectorize(builder.result());
                }
                false
            },
            Stage::Vectorize(clusters) => {
                let view = clusters.view();
                if self.counter < view.clusters_output.len() {
                    if let Some(canvas) = &self.canvas {
                        canvas.log("Vectorize tick");
                    }
                    let cluster = view.get_cluster(view.clusters_output[self.counter]);
                    let paths = cluster.to_compound_path(
                        &view,
                        false,
                        self.mode,
                        self.params.corner_threshold,
                        self.params.length_threshold,
                        self.params.max_iterations,
                        self.params.splice_threshold,
                    );
                    if let Some(svg) = &mut self.svg {
                        svg.prepend_path(&paths, &cluster.residue_color(), Some(self.params.path_precision));
                    }
                    self.counter += 1;
                    false
                } else {
                    if let Some(canvas) = &self.canvas {
                        canvas.log("done");
                    }
                    true
                }
            }
        }
    }

    pub fn progress(&self) -> i32 {
        let progress = match &self.stage {
            Stage::New => 0,
            Stage::Clustering(builder) => builder.progress() / 2,
            Stage::Reclustering(_) => 50,
            Stage::Vectorize(clusters) => {
                50 + 50 * self.counter as u32 / clusters.view().clusters_output.len() as u32
            }
        };
        progress as i32
    }

    fn color_exists_in_image(img: &ColorImage, color: Color) -> bool {
        for y in 0..img.height {
            for x in 0..img.width {
                let p = img.get_pixel(x, y);
                if p.r == color.r && p.g == color.g && p.b == color.b {
                    return true;
                }
            }
        }
        false
    }

    fn find_unused_color_in_image(img: &ColorImage) -> Result<Color, String> {
        let candidates = [
            Color::new(255, 0, 0),
            Color::new(0, 255, 0),
            Color::new(0, 0, 255),
            Color::new(255, 255, 0),
            Color::new(0, 255, 255),
            Color::new(255, 0, 255),
            Color::new(128, 128, 128),
        ];
        for color in candidates {
            if !Self::color_exists_in_image(img, color) {
                return Ok(color);
            }
        }
        Err("unable to find unused color in image to use as key".into())
    }

    fn should_key_image(img: &ColorImage) -> bool {
        if img.width == 0 || img.height == 0 {
            return false;
        }
        let threshold = ((img.width * 2) as f32 * KEYING_THRESHOLD) as usize;
        let mut count = 0;
        for &y in &[0, img.height / 4, img.height / 2, 3 * img.height / 4, img.height - 1] {
            for x in 0..img.width {
                if img.get_pixel(x, y).a == 0 {
                    count += 1;
                    if count >= threshold {
                        return true;
                    }
                }
            }
        }
        false
    }
}
