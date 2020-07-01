use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_longlong};
use std::ptr::{null, null_mut};

use encoding_rs::UTF_8;

use crate::common::NativeByteArray;

#[repr(C)]
pub struct TextEncoder {
    encoder: &'static encoding_rs::Encoding,
}

impl TextEncoder {
    pub fn new(encoding: *const c_char) -> Self {
        let encoding = unsafe { CStr::from_ptr(encoding) }
            .to_str()
            .unwrap_or("utf-8");
        let encoder = encoding_rs::Encoding::for_label(encoding.as_bytes())
            .unwrap_or(UTF_8.output_encoding());
        Self { encoder }
    }

    pub fn encode(&mut self, text: *const c_char) -> NativeByteArray {
        let txt = unsafe { CStr::from_ptr(text) }.to_str().unwrap_or("");
        let result = self.encoder.encode(txt);
        let mut data = Vec::from(result.0).into_boxed_slice();
        let array = NativeByteArray {
            array: data.as_mut_ptr(),
            length: data.len(),
        };
        mem::forget(data);
        array
    }

    pub fn encoding(&self) -> *const c_char {
        CString::new(self.encoder.name()).unwrap().into_raw()
    }

    pub fn release(ptr: c_longlong) {
        if ptr != 0 {
            let _: Box<TextEncoder> = unsafe { Box::from_raw(ptr as *mut _) };
        }
    }
}

pub(crate) fn text_encoder_get_encoding(encoder: c_longlong) -> *const c_char {
    if encoder != 0 {
        let encoder: Box<TextEncoder> = unsafe { Box::from_raw(encoder as *mut _) };
        return encoder.encoding();
    }
    null()
}

pub(crate) fn text_encoder_encode(encoder: c_longlong, text: *const c_char) -> NativeByteArray {
    if encoder != 0 {
        let mut encoder: Box<TextEncoder> = unsafe { Box::from_raw(encoder as *mut _) };
        let buffer = encoder.encode(text);
        Box::into_raw(encoder);
        return buffer;
    }
    NativeByteArray {
        array: null_mut(),
        length: 0,
    }
}

pub(crate) fn free_byte_array(array: NativeByteArray) {
    if array.array.is_null() || array.length == 0 {
        return;
    }
    let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(array.array, array.length)) };
}

pub(crate) fn free_text_encoder(encoder: i64) {
    TextEncoder::release(encoder);
}
