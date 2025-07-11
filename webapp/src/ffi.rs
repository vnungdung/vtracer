use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_uchar};
use std::slice;

use crate::conversion::binary_image::{BinaryImageConverter, BinaryImageConverterParams};
use crate::conversion::color_image::{ColorImageConverter, ColorImageConverterParams};

// ===========================
// === BinaryImageConverter ===
// ===========================

#[no_mangle]
pub extern "C" fn vtracer_binary_init(json_params: *const c_char) -> *mut BinaryImageConverter {
    let c_str = unsafe { CStr::from_ptr(json_params) };
    let json = c_str.to_str().unwrap();
    let params: BinaryImageConverterParams = serde_json::from_str(json).unwrap();
    let converter = BinaryImageConverter::new(params);
    Box::into_raw(Box::new(converter))
}

#[no_mangle]
pub extern "C" fn vtracer_binary_from_bytes(
    json_params: *const c_char,
    data: *const c_uchar,
    len: usize,
    width: usize,
    height: usize,
) -> *mut BinaryImageConverter {
    let c_str = unsafe { CStr::from_ptr(json_params) };
    let json = c_str.to_str().unwrap();
    let params: BinaryImageConverterParams = serde_json::from_str(json).unwrap();

    let bytes = unsafe { slice::from_raw_parts(data, len) };
    let converter = BinaryImageConverter::from_bytes(bytes, width, height, params);
    Box::into_raw(Box::new(converter))
}

#[no_mangle]
pub extern "C" fn vtracer_binary_init_state(ptr: *mut BinaryImageConverter) {
    if !ptr.is_null() {
        let converter = unsafe { &mut *ptr };
        converter.init();
    }
}

#[no_mangle]
pub extern "C" fn vtracer_binary_tick(ptr: *mut BinaryImageConverter) -> bool {
    if ptr.is_null() {
        return true;
    }
    let converter = unsafe { &mut *ptr };
    converter.tick()
}

#[no_mangle]
pub extern "C" fn vtracer_binary_progress(ptr: *mut BinaryImageConverter) -> c_int {
    if ptr.is_null() {
        return 100;
    }
    let converter = unsafe { &*ptr };
    converter.progress() as c_int
}

#[no_mangle]
pub extern "C" fn vtracer_binary_free(ptr: *mut BinaryImageConverter) {
    if !ptr.is_null() {
        unsafe { Box::from_raw(ptr); }
    }
}

// ===========================
// === ColorImageConverter ===
// ===========================

#[no_mangle]
pub extern "C" fn vtracer_color_init(json_params: *const c_char) -> *mut ColorImageConverter {
    let c_str = unsafe { CStr::from_ptr(json_params) };
    let json = c_str.to_str().unwrap();
    let params: ColorImageConverterParams = serde_json::from_str(json).unwrap();
    let converter = ColorImageConverter::new(params);
    Box::into_raw(Box::new(converter))
}

#[no_mangle]
pub extern "C" fn vtracer_color_init_state(ptr: *mut ColorImageConverter) {
    if !ptr.is_null() {
        let converter = unsafe { &mut *ptr };
        converter.init();
    }
}

#[no_mangle]
pub extern "C" fn vtracer_color_from_bytes(
    json_params: *const c_char,
    data: *const c_uchar,
    len: usize,
    width: usize,
    height: usize,
) -> *mut ColorImageConverter {
    let c_str = unsafe { CStr::from_ptr(json_params) };
    let json = c_str.to_str().unwrap();
    let params: ColorImageConverterParams = serde_json::from_str(json).unwrap();

    let bytes = unsafe { slice::from_raw_parts(data, len) };
    let converter = ColorImageConverter::from_bytes(bytes, width, height, params);
    Box::into_raw(Box::new(converter))
}

#[no_mangle]
pub extern "C" fn vtracer_color_tick(ptr: *mut ColorImageConverter) -> bool {
    if ptr.is_null() {
        return true;
    }
    let converter = unsafe { &mut *ptr };
    converter.tick()
}

#[no_mangle]
pub extern "C" fn vtracer_color_progress(ptr: *mut ColorImageConverter) -> c_int {
    if ptr.is_null() {
        return 100;
    }
    let converter = unsafe { &*ptr };
    converter.progress()
}

#[no_mangle]
pub extern "C" fn vtracer_color_free(ptr: *mut ColorImageConverter) {
    if !ptr.is_null() {
        unsafe { Box::from_raw(ptr); }
    }
}
