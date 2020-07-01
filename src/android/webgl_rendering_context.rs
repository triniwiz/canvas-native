#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
extern crate libc;

use std::mem;
use std::os::raw::c_void;
use std::ptr::null_mut;

use jni::{
    JNIEnv,
    objects::{JClass, JObject},
};
use jni_sys::{jboolean, jbyteArray, jint, JNI_TRUE, jobject};
use log::{debug};

use crate::android::bitmap::{
    ANDROID_BITMAP_RESULT_SUCCESS, AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels,
    AndroidBitmapInfo,
};
use jni::objects::JByteBuffer;

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_WebGLRenderingContext_nativeFlipInPlace3D(env: JNIEnv, _: JClass, pixels: jbyteArray, width: jint, height: jint, depth: jint) {
    self::super::core::flip_in_place_3d(env, pixels, width, height, depth);
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_WebGLRenderingContext_nativeFlipInPlace(env: JNIEnv, _: JClass, pixels: jbyteArray, width: jint, height: jint) {
    self::super::core::flip_in_place(env, pixels, width, height);
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_WebGLRenderingContext_nativeBytesFromBitmap(env: JNIEnv, _: JClass, bitmap: JObject, flipY: jboolean) -> jbyteArray {
    let native_interface = env.get_native_interface();
    let bitmap_raw = bitmap.into_inner();
    let bitmap_info = Box::into_raw(Box::new(AndroidBitmapInfo::default()));

    if AndroidBitmap_getInfo(native_interface, bitmap_raw, bitmap_info)
        < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("bytesFromBitmap get bitmap info failed");
        return env.new_byte_array(0).unwrap();
    }
    let info_to_draw: Box<AndroidBitmapInfo> = Box::from_raw(bitmap_info);

    let mut _dstPixelsToDraw = null_mut() as *mut c_void;
    let dstPixelsToDraw: *mut *mut c_void = &mut _dstPixelsToDraw;
    if AndroidBitmap_lockPixels(native_interface, bitmap_raw, dstPixelsToDraw)
        < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("bytesFromBitmap get bitmap lock failed");
        return env.new_byte_array(0).unwrap();
    }
    let ratio_to_draw = mem::size_of_val(&dstPixelsToDraw) / mem::size_of::<u8>();
    let length_to_draw =
        ((info_to_draw.width * info_to_draw.height) * ratio_to_draw as u32) as usize;

    let ptr_to_draw = _dstPixelsToDraw as *mut _;
    let mut pixels_to_draw: &mut [i8] =
        std::slice::from_raw_parts_mut(ptr_to_draw as *mut _, length_to_draw as usize);
    let mut storage;
    if flipY == JNI_TRUE {
        let width = info_to_draw.width;
        let height = info_to_draw.height;
        let line_size = width * 4;
        let mut line_buffer_storage = vec![0i8; line_size as usize];
        let mut line_buffer = line_buffer_storage.as_mut_ptr();
        let mut data_storage = pixels_to_draw;
        let data = data_storage.as_mut_ptr();
        let half_height = height / 2;
        for y in 0..half_height {
            let top_line = data.offset((y * line_size) as isize);
            let bottom_line = data.offset(((height - y - 1) * line_size) as isize);
            std::ptr::copy_nonoverlapping(top_line, line_buffer, line_size as usize);
            std::ptr::copy_nonoverlapping(bottom_line, top_line, line_size as usize);
            std::ptr::copy_nonoverlapping(bottom_line, line_buffer, line_size as usize);
        }

        let storage_slice = { &*(data_storage as *mut [i8] as *mut [u8]) };
        storage = env.byte_array_from_slice(storage_slice).unwrap();
    } else {
        let storage_slice = { &*(pixels_to_draw as *mut [i8] as *mut [u8]) };
        storage = env.byte_array_from_slice(storage_slice).unwrap();
    }


    if AndroidBitmap_unlockPixels(native_interface, bitmap_raw) < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("bytesFromBitmap unlock bitmap failed");
    }

    storage
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_WebGLRenderingContext_nativeGetVertexAttribOffset(env: JNIEnv, _: JClass, index: jint, pname: jint, buffer: JByteBuffer) {
    let buf = env.get_direct_buffer_address(buffer).unwrap();
    let mut ptr = buf.as_ptr() as *mut c_void;
    let ptr_ptr: *mut *mut c_void = &mut ptr;
    crate::android::gl::glGetVertexAttribPointerv(index as std::os::raw::c_uint, pname as std::os::raw::c_uint, ptr_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_WebGLRenderingContext_nativeBindBuffer(_env: JNIEnv, _: JClass, target: jint, buffer: jint) {
    crate::android::gl::glBindBuffer(target as std::os::raw::c_uint, buffer as std::os::raw::c_uint);
}