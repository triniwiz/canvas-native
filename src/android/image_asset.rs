#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
extern crate libc;

use std::ffi::CStr;

use jni::{
    JNIEnv,
    objects::{JClass, JString}
};
use jni_sys::{jboolean, jbyteArray, jint, jlong, jstring};

use crate::common::{create_image_asset, image_asset_flip_x, image_asset_flip_y, image_asset_free_bytes, image_asset_get_bytes, image_asset_get_error, image_asset_height, image_asset_load_from_path, image_asset_load_from_slice_i8, image_asset_release, image_asset_save_path, image_asset_scale, image_asset_width};

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeInit(_env: JNIEnv, _: JClass) -> jlong {
    create_image_asset()
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeGetBytes(env: JNIEnv, _: JClass, asset: jlong) -> jbyteArray {
    let mut array = image_asset_get_bytes(asset);
    let bytes = std::slice::from_raw_parts(array.array as *const u8, array.length);
    let result = env.byte_array_from_slice(bytes).unwrap_or(env.new_byte_array(0).unwrap());
    image_asset_free_bytes(array);
    result
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeGetWidth(_env: JNIEnv, _: JClass, asset: jlong) -> jint {
    image_asset_width(asset) as i32
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeGetHeight(_env: JNIEnv, _: JClass, asset: jlong) -> jint {
    image_asset_height(asset) as i32
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeScale(_env: JNIEnv, _: JClass, asset: jlong, x: jint, y: jint) -> jlong {
    image_asset_scale(asset, x as u32, y as u32)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeFlipX(_env: JNIEnv, _: JClass, asset: jlong) -> jlong {
    image_asset_flip_x(asset)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeSave(env: JNIEnv, _: JClass, asset: jlong, path: JString, format: jint) -> jboolean {
    let real_path = env.get_string(path).unwrap();
    image_asset_save_path(asset, real_path.get_raw(), format as u32) as u8
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeFlipY(_env: JNIEnv, _: JClass, asset: jlong) -> jlong {
    image_asset_flip_y(asset)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeGetError(env: JNIEnv, _: JClass, asset: jlong) -> jstring {
    let error = image_asset_get_error(asset);
    let string = CStr::from_ptr(error).to_str();
    let string = string.unwrap_or("");
    env.new_string(string).unwrap().into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeRelease(_env: JNIEnv, _: JClass, asset: jlong) {
    image_asset_release(asset)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeLoadAssetPath(env: JNIEnv, _: JClass, asset: jlong, path: JString) -> jboolean {
    let real_path = env.get_string(path).unwrap();
    image_asset_load_from_path(asset, real_path.get_raw()) as u8
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_ImageAsset_nativeLoadAssetBuffer(env: JNIEnv, _: JClass, asset: jlong, buffer: jbyteArray) -> jboolean {
    let size = env.get_array_length(buffer).unwrap_or(0);
    let mut buf = vec![0i8; size as usize];
    let _ = env.get_byte_array_region(buffer, 0, buf.as_mut_slice());
    image_asset_load_from_slice_i8(asset, buf.as_mut_slice()) as u8
}
