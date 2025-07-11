use wasm_bindgen::prelude::*;
use visioncortex::{clusters::Clusters, Color, ColorImage, ColorName, PathSimplifyMode};

use crate::{canvas::*, svg::*};
use serde::Deserialize;
use super::util;

#[derive(Debug, Deserialize, Clone)]
pub struct BinaryImageConverterParams {
    pub canvas_id: String,
    pub svg_id: String,
    pub mode: String,
    pub corner_threshold: f64,
    pub length_threshold: f64,
    pub max_iterations: usize,
    pub splice_threshold: f64,
    pub filter_speckle: usize,
    pub path_precision: u32,
}

#[wasm_bindgen]
pub struct BinaryImageConverter {
    canvas: Option<Canvas>,
    svg: Option<Svg>,
    clusters: Clusters,
    counter: usize,
    mode: PathSimplifyMode,
    params: BinaryImageConverterParams,
}

impl BinaryImageConverter {
    /// Sử dụng DOM canvas như trước
    pub fn new(params: BinaryImageConverterParams) -> Self {
        let canvas = Canvas::new_from_id(&params.canvas_id);
        let svg = Svg::new_from_id(&params.svg_id);
        Self {
            canvas: Some(canvas),
            svg: Some(svg),
            clusters: Clusters::default(),
            counter: 0,
            mode: util::path_simplify_mode(&params.mode),
            params,
        }
    }

    /// Khởi tạo từ byte buffer RGBA
    pub fn from_bytes(bytes: &[u8], width: usize, height: usize, params: BinaryImageConverterParams) -> Self {
        let image = ColorImage {
            pixels: bytes.to_vec(),
            width,
            height,
        };
        let binary_image = image.to_binary_image(|x| x.r < 128);
        let clusters = binary_image.to_clusters(false);
        Self {
            canvas: None,
            svg: None,
            clusters,
            counter: 0,
            mode: util::path_simplify_mode(&params.mode),
            params,
        }
    }
}

#[wasm_bindgen]
impl BinaryImageConverter {
    pub fn new_with_string(params: String) -> Self {
        let params: BinaryImageConverterParams = serde_json::from_str(params.as_str()).unwrap();
        Self::new(params)
    }

    pub fn init(&mut self) {
        if let Some(canvas) = &self.canvas {
            let width = canvas.width() as u32;
            let height = canvas.height() as u32;
            let image = canvas.get_image_data_as_color_image(0, 0, width, height);
            let binary_image = image.to_binary_image(|x| x.r < 128);
            self.clusters = binary_image.to_clusters(false);
            canvas.log(&format!(
                "clusters.len() = {}, self.clusters.rect.left = {}",
                self.clusters.len(),
                self.clusters.rect.left
            ));
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.counter < self.clusters.len() {
            if let Some(canvas) = &self.canvas {
                canvas.log(&format!("tick {}", self.counter));
            }
            let cluster = self.clusters.get_cluster(self.counter);
            if cluster.size() >= self.params.filter_speckle {
                let paths = cluster.to_compound_path(
                    self.mode,
                    self.params.corner_threshold,
                    self.params.length_threshold,
                    self.params.max_iterations,
                    self.params.splice_threshold,
                );
                let color = Color::color(&ColorName::Black);
                if let Some(svg) = &mut self.svg {
                    svg.prepend_path(&paths, &color, Some(self.params.path_precision));
                }
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

    pub fn progress(&self) -> u32 {
        if self.clusters.len() == 0 {
            100
        } else {
            100 * self.counter as u32 / self.clusters.len() as u32
        }
    }
}
