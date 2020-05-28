#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
extern crate libc;

use std::borrow::{Borrow, BorrowMut};
use std::ffi::{CStr, CString};
use std::fs::Metadata;
use std::io::{Error, Read};
use std::mem;
use std::ops::Deref;
use std::os::raw::{c_char, c_float, c_longlong, c_void};
use std::ptr::null_mut;
use std::string::String;

use android_logger::Config;
use jni::{JNIEnv, sys};
use jni::objects::{JClass, JObject, JString, JValue};
use jni::strings::JavaStr;
use jni::sys::{jboolean, jint, jintArray, jlong, jobject, jstring};
use jni_sys::{jbyte, jbyteArray, jfloat, jfloatArray, JNI_FALSE, JNI_TRUE};
use libc::{c_int, size_t};
use log::{debug, info};
use log::Level;
use skia_safe::{AlphaType, Bitmap, BlendMode, Canvas, Color, ColorSpace, ColorType, Data, FilterQuality, Font, FontMetrics, FontStyle, FontStyleWeight, FontStyleWidth, Image, ImageInfo, IPoint, ISize, MaskFilter, Paint, Path, PixelGeometry, Pixmap, Point, Rect, Size, Surface, SurfaceProps, SurfacePropsFlags, TextBlob, Typeface};
use skia_safe::gpu::{BackendRenderTarget, Context, gl, ResourceCacheLimits, SurfaceOrigin};
use skia_safe::gpu::gl::Interface;
use skia_safe::image_filters::image;
use skia_safe::paint::{Cap, Join, Style};
use skia_safe::path::Direction;

use crate::android::bitmap::{
    ANDROID_BITMAP_RESULT_SUCCESS, AndroidBitmap_getInfo, AndroidBitmap_lockPixels, AndroidBitmap_unlockPixels,
    AndroidBitmapInfo,
};
use crate::common::{add_path_to_path, add_path_to_path_with_matrix, adjust_end_angle, arc, arc_to, begin_path, bezier_curve_to, CanvasNative, CanvasState, CanvasStateItem, clear_canvas, clear_rect, clip, clip_path_rule, clip_rule, close_path, create_image_asset, create_image_data, create_matrix, create_path_2d_from_path_data, create_path_from_path, draw_image, draw_image_dw, draw_image_sw, draw_rect, draw_svg_image, draw_text, ellipse, ellipse_no_rotation, fill, fill_path_rule, fill_rule, flush, free_matrix, free_path_2d, get_current_transform, get_image_data, get_matrix, get_measure_text, image_asset_flip_x, image_asset_flip_y, image_asset_flip_y_in_place_owned, image_asset_free_bytes, image_asset_get_bytes, image_asset_get_error, image_asset_height, image_asset_load_from_path, image_asset_load_from_raw, image_asset_load_from_slice_i8, image_asset_release, image_asset_save_path, image_asset_scale, image_asset_width, is_font_size, is_font_style, is_font_weight, is_point_in_path, is_point_in_stroke, line_to, move_to, NativeImageAsset, put_image_data, quadratic_curve_to, rect, reset_transform, restore, rotate, save, scale, set_current_transform, set_fill_color, set_fill_color_rgba, set_font, set_global_alpha, set_global_composite_operation, set_gradient_linear, set_gradient_radial, set_image_smoothing_enabled, set_image_smoothing_quality, set_line_cap, set_line_dash, set_line_dash_offset, set_line_join, set_line_width, set_matrix, set_miter_limit, set_shadow_blur, set_shadow_color, set_shadow_offset_x, set_shadow_offset_y, set_stroke_color, set_stroke_color_rgba, set_text_align, set_transform, stroke, stroke_path, SVGCanvasNative, text_decoder_decode, text_decoder_get_encoding, text_encoder_encode, text_encoder_get_encoding, TextDecoder, TextEncoder, to_byte_slice, to_data, to_data_url, transform, translate};

pub const COLOR_BLACK: usize = 0xff000000 as usize;

#[no_mangle]
pub extern "system" fn JNI_OnLoad() -> jint {
    {
        android_logger::init_once(Config::default().with_min_level(Level::Debug));
        info!("Canvas Native library loaded");
    }

    jni::sys::JNI_VERSION_1_6
}


unsafe fn drawText(
    env: JNIEnv,
    canvas_native_ptr: jlong,
    text: JString,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    is_stoke: bool,
) -> jlong {
    draw_text(
        canvas_native_ptr,
        env.get_string(text).unwrap().as_ptr() as _,
        x,
        y,
        width,
        is_stoke,
    )
}

unsafe fn drawRect(
    _env: JNIEnv,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
    is_stoke: bool,
) -> jlong {
    draw_rect(canvas_native_ptr, x, y, width, height, is_stoke)
}

fn init(buffer_id: jint,
        width: jint,
        height: jint,
        scale: jfloat, ) -> CanvasNative {
    let mut stroke_paint = Paint::default();
    stroke_paint.set_anti_alias(true);
    stroke_paint.set_color(Color::from(COLOR_BLACK as u32));
    stroke_paint.set_stroke_width(1.0);
    stroke_paint.set_style(Style::Stroke);
    stroke_paint.set_stroke_join(Join::Miter);
    stroke_paint.set_stroke_cap(Cap::Butt);
    stroke_paint.set_stroke_miter(10.0);
    let mut fill_paint = Paint::default();
    fill_paint.set_anti_alias(true);
    fill_paint.set_color(Color::from(COLOR_BLACK as u32));
    fill_paint.set_style(Style::Fill);
    fill_paint.set_stroke_miter(10.0);
    fill_paint.set_stroke_join(Join::Miter);
    fill_paint.set_stroke_cap(Cap::Butt);
    // "10px sans-serif" Default
    let default_type_face =
        Typeface::from_name("sans-serif", FontStyle::default()).unwrap_or(Typeface::default());
    let mut font = Font::from_typeface(&default_type_face, Some(10.0));
    let state: Vec<CanvasStateItem> = Vec::new();

    let interface = gl::Interface::new_native();
    let context = Context::new_gl(interface.unwrap());
    let mut ctx = context.unwrap();
    let mut k_gr_cache_max_count = 8192;
    let mut _k_gr_cache_max_byte_size = 24 * (1 << 20);
    let max_bytes = width * height * 12 * 5;
    //ctx.set_resource_cache_limits(ResourceCacheLimits { max_resources: k_gr_cache_max_count, max_resource_bytes: max_bytes as usize });
    let mut frame_buffer = gl::FramebufferInfo::from_fboid(buffer_id as u32);
    frame_buffer.format = 0x8058; //GR_GL_RGBA8 (https://github.com/google/skia/blob/master/src/gpu/gl/GrGLDefines.h#L511)
    let target =
        BackendRenderTarget::new_gl((width as i32, height as i32), Some(0), 8, frame_buffer);
    let surface_props = SurfaceProps::new(SurfacePropsFlags::default(), PixelGeometry::Unknown);
    let color_space = ColorSpace::new_srgb();
    let surface_holder = Surface::from_backend_render_target(
        &mut ctx,
        &target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        Some(color_space),
        Some(&surface_props),
    );
    let mut surface = surface_holder.unwrap();
    let mut native_canvas = CanvasNative {
        surface,
        stroke_paint,
        fill_paint,
        path: Path::new(),
        context: Some(ctx),
        font,
        state,
        line_dash_offset: 0.0,
        shadow_blur: 0.0,
        shadow_color: COLOR_BLACK as u32,
        shadow_offset_x: 0.0,
        shadow_offset_y: 0.0,
        image_smoothing_enabled: false,
        image_smoothing_quality: "low".to_string(),
        device_scale: scale,
        text_align: "left".to_string(),
        ios: 0,
    };

    native_canvas
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_FileReader_nativeRead(env: JNIEnv, _: JClass, file: JString) -> jbyteArray {
    let empty = env.new_string("").unwrap();
    let mut value = env.get_string(file).unwrap_or(JavaStr::from_env(&env, empty).unwrap());
    let mut real_file = std::fs::File::open(std::path::Path::new(value.to_str().unwrap()));
    let result = match real_file {
        Ok(mut file) => {
            let len: usize = match file.metadata() {
                Ok(len) => len.len(),
                Err(err) => {
                    0
                }
            } as usize;
            let mut data = vec![0u8; len];
            file.read_to_end(&mut data);
            data
        }
        Err(e) => {
            Vec::new()
        }
    };

    env.byte_array_from_slice(result.as_slice()).unwrap()
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_TextEncoder_nativeInit(env: JNIEnv, _: JClass, encoding: JString) -> jlong {
    let empty = env.new_string("").unwrap();
    let mut value = env.get_string(encoding).unwrap_or(JavaStr::from_env(&env, empty).unwrap());
    Box::into_raw(Box::new(TextEncoder::new(value.get_raw()))) as i64
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_TextEncoder_nativeGetEncoding(env: JNIEnv, _: JClass, encoder: i64) -> jstring {
    let encoding = text_encoder_get_encoding(encoder);
    let value = CStr::from_ptr(encoding).to_str().unwrap_or("");
    env.new_string(value).unwrap().into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_TextEncoder_nativeEncode(env: JNIEnv, _: JClass, encoder: i64, text: JString) -> jbyteArray {
    let empty = env.new_string("").unwrap();
    let mut string = env.get_string(text).unwrap_or(JavaStr::from_env(&env, empty).unwrap());
    let rawArray = text_encoder_encode(encoder, string.as_ptr());
    let rawSlice = std::slice::from_raw_parts_mut(rawArray.array, rawArray.length);
    let javaArray = env.byte_array_from_slice(rawSlice).unwrap();
    let _ = Box::from_raw(rawSlice).to_vec();
    javaArray
}


#[no_mangle]
pub extern "C" fn Java_com_github_triniwiz_canvas_TextDecoder_nativeInit(env: JNIEnv, _: JClass, decoding: JString) -> jlong {
    let empty = env.new_string("").unwrap();
    let mut value = env.get_string(decoding).unwrap_or(JavaStr::from_env(&env, empty).unwrap());
    Box::into_raw(Box::new(TextDecoder::new(value.get_raw()))) as i64
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_TextDecoder_nativeGetEncoding(env: JNIEnv, _: JClass, decoder: i64) -> jstring {
    let encoding = text_decoder_get_encoding(decoder);
    let value = CStr::from_ptr(encoding).to_str().unwrap_or("");
    env.new_string(value).unwrap().into_inner()
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_TextDecoder_nativeDecode(env: JNIEnv, _: JClass, decoder: i64, data: jbyteArray) -> jstring {
    let len = env.get_array_length(data).unwrap_or(0);
    let mut rawData = vec![0i8; len as usize];
    env.get_byte_array_region(data, 0, rawData.as_mut_slice());
    let buf = unsafe { &*(rawData.as_mut_slice() as *mut [i8] as *mut [u8]) };
    let decoded = text_decoder_decode(decoder, buf.as_ptr(), rawData.len());
    let value = CStr::from_ptr(decoded).to_str().unwrap_or("");
    env.new_string(value).unwrap().into_inner()
}


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
    env.new_string(string.unwrap_or("")).unwrap().into_inner()
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
    env.get_byte_array_region(buffer, 0, buf.as_mut_slice());
    image_asset_load_from_slice_i8(asset, buf.as_mut_slice()) as u8
}


unsafe fn flip_in_place_3d(env: JNIEnv, pixels: jbyteArray, width: jint, height: jint, depth: jint) {
    let size = env.get_array_length(pixels).unwrap_or(0);
    let mut array = vec![0i8; size as usize];
    env.get_byte_array_region(pixels, 0, array.as_mut_slice());
    let mut data = array.as_mut_ptr();
    for _z in 0..depth {
        flip_in_place_native(data, width, height);
        data.offset((4 * width * height) as isize);
    }
    env.set_byte_array_region(pixels, 0, array.as_mut_slice());
}

unsafe fn flip_in_place_native(pixels: *mut i8, width: i32, height: i32) {
    image_asset_flip_y_in_place_owned(width as u32, height as u32, pixels as *mut u8, (width * height * 4) as usize);
}


unsafe fn flip_in_place(env: JNIEnv, pixels: jbyteArray, width: jint, height: jint) {
    let line_size = width * 4;
    let mut line_buffer_storage = vec![0i8; line_size as usize];
    let mut _line_buffer = line_buffer_storage.as_mut_ptr();
    let storage_size = env.get_array_length(pixels).unwrap_or(0);
    let mut data_storage = vec![0i8; storage_size as usize];
    env.get_byte_array_region(pixels, 0, data_storage.as_mut_slice());
    let data = data_storage.as_mut_ptr();

    image_asset_flip_y_in_place_owned(width as u32, height as u32, data as *mut u8, storage_size as usize);

    let flipped = std::slice::from_raw_parts(data_storage.as_ptr(), data_storage.len());
    env.set_byte_array_region(pixels, 0, flipped);
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_WebGLRenderingContext_nativeFlipInPlace3D(env: JNIEnv, _: JClass, pixels: jbyteArray, width: jint, height: jint, depth: jint) {
    flip_in_place_3d(env, pixels, width, height, depth);
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_WebGLRenderingContext_nativeFlipInPlace(env: JNIEnv, _: JClass, pixels: jbyteArray, width: jint, height: jint) {
    flip_in_place(env, pixels, width, height);
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
            let top_line = data.offset(((y * line_size) as isize));
            let bottom_line = data.offset((((height - y - 1) * line_size) as isize));
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
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasView_nativeDestroy(
    _env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) {
    let mut canvas: CanvasNative = *Box::from_raw(canvas_native_ptr as *mut _);
    let mut ctx = canvas.context.unwrap();
    ctx.abandon();
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasView_nativeInit(
    _env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    buffer_id: jint,
    width: jint,
    height: jint,
    scale: jfloat,
) -> jlong {
    Box::into_raw(Box::new(init(buffer_id, width, height, scale))) as *mut _ as i64
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasView_nativeResize(
    _env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    buffer_id: jint,
    width: jint,
    height: jint,
    scale: jfloat,
) -> jlong {
    if canvas_native_ptr == 0 {
        return canvas_native_ptr;
    }
    let mut canvas_native: Box<CanvasNative> = Box::from_raw(canvas_native_ptr as *mut _);
    let mut surface = &mut canvas_native.surface;
    let mut ctx = canvas_native.context.unwrap();
    surface.flush();
    let mut frame_buffer = gl::FramebufferInfo::from_fboid(buffer_id as u32);
    frame_buffer.format = 0x8058; //GR_GL_RGBA8 (https://github.com/google/skia/blob/master/src/gpu/gl/GrGLDefines.h#L511)
    let target =
        BackendRenderTarget::new_gl((width as i32, height as i32), Some(0), 8, frame_buffer);
    let surface_props = SurfaceProps::new(SurfacePropsFlags::default(), PixelGeometry::Unknown);
    let color_space = ColorSpace::new_srgb();
    let surface_holder = Surface::from_backend_render_target(
        &mut ctx,
        &target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        Some(color_space),
        Some(&surface_props),
    );
    let mut surface = surface_holder.unwrap();
    canvas_native.surface = surface;
    canvas_native.context = Some(ctx);
    Box::into_raw(canvas_native) as *mut _ as i64
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasView_nativeRecreate(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    buffer_id: jint,
    width: jint,
    height: jint,
    scale: jfloat,
) -> jlong {
    if canvas_native_ptr < 0 {
        return canvas_native_ptr;
    }
    let mut canvas_native: Box<CanvasNative> = Box::from_raw(canvas_native_ptr as *mut _);
    let mut ctx = canvas_native.context.unwrap();
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    canvas.flush();
    surface.flush();
    let mut ss = surface.image_snapshot();
    let mut interface = gl::Interface::new_native();
    let mut ctx = Context::new_gl(interface.unwrap()).unwrap();
    let mut frame_buffer = gl::FramebufferInfo::from_fboid(buffer_id as u32);
    frame_buffer.format = 0x8058; //GR_GL_RGBA8 (https://github.com/google/skia/blob/master/src/gpu/gl/GrGLDefines.h#L511)
    let target =
        BackendRenderTarget::new_gl((width, height), Some(0), 8, frame_buffer);
    let surface_props = SurfaceProps::new(SurfacePropsFlags::default(), PixelGeometry::Unknown);
    let color_space = ColorSpace::new_srgb();
    let surface_holder = Surface::from_backend_render_target(
        &mut ctx,
        &target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        Some(color_space),
        Some(&surface_props),
    );
    let mut new_surface = surface_holder.unwrap();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    new_surface.canvas().draw_image(ss, Point::new(0f32, 0f32), Some(&paint));
    paint.set_color(Color::RED);
    new_surface.canvas().draw_rect(Rect::new(0f32, 0f32, width as f32, height as f32), &paint);
    new_surface.canvas().flush();
    new_surface.flush();
    canvas_native.surface = new_surface;
    canvas_native.context = Some(ctx);
    Box::into_raw(Box::new(canvas_native)) as *mut _ as i64
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasView_nativeFlush(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jlong {
    flush(canvas_native_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasView_nativeToData(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jbyteArray {
    let mut data = to_data(canvas_native_ptr);
    env.byte_array_from_slice(data.as_mut_slice())
        .unwrap()
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasView_nativeToDataUrl(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    format: JString,
    quality: jfloat,
) -> jstring {
    let default = env.new_string("image/png").unwrap();
    let javaStr = JavaStr::from_env(&env, default);
    let format = env.get_string(format).unwrap_or(javaStr.unwrap());
    let result = to_data_url(canvas_native_ptr, format.as_ptr(), ((quality * 100f32) as i32));
    let string = CStr::from_ptr(result).to_str();
    env.new_string(string.unwrap()).unwrap().into_inner()
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetMiterLimit(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    limit: f32,
) -> jlong {
    set_miter_limit(canvas_native_ptr, limit)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeArc(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    radius: jfloat,
    start_angle: jfloat,
    end_angle: jfloat,
    anticlockwise: jboolean,
) -> jlong {
    arc(
        canvas_native_ptr,
        true,
        x,
        y,
        radius,
        start_angle,
        end_angle,
        anticlockwise == JNI_TRUE,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeArcTo(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x1: jfloat,
    y1: jfloat,
    x2: jfloat,
    y2: jfloat,
    radius: jfloat,
) -> jlong {
    arc_to(canvas_native_ptr, true, x1, y1, x2, y2, radius)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeBeginPath(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jlong {
    begin_path(canvas_native_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeBezierCurveTo(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    cp1x: jfloat,
    cp1y: jfloat,
    cp2x: jfloat,
    cp2y: jfloat,
    x: jfloat,
    y: jfloat,
) -> jlong {
    bezier_curve_to(canvas_native_ptr, true, cp1x, cp1y, cp2x, cp2y, x, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeClearRect(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jlong {
    clear_rect(canvas_native_ptr, x, y, width, height)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeClipPathRule(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    path: jlong,
    fill_rule: JString,
) -> jlong {
    clip_path_rule(canvas_native_ptr, path, env.get_string(fill_rule).unwrap().as_ptr() as _)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeClip(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jlong {
    clip(
        canvas_native_ptr
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeClipRule(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    fill_rule: JString,
) -> jlong {
    clip_rule(
        canvas_native_ptr,
        env.get_string(fill_rule).unwrap().as_ptr() as _,
    )
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeClosePath(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jlong {
    close_path(canvas_native_ptr, true)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetFillColorRgba(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
) -> jlong {
    set_fill_color_rgba(canvas_native_ptr, red, green, blue, alpha)
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetFillColor(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    color: jint,
) -> jlong {
    set_fill_color(canvas_native_ptr, color as u32)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetStrokeColorRgba(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
) -> jlong {
    set_stroke_color_rgba(canvas_native_ptr, red, green, blue, alpha)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetStrokeColor(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    color: jint,
) -> jlong {
    set_stroke_color(canvas_native_ptr, color as u32)
}

// set from createLinearGradient()

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetFillGradientLinear(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x0: jfloat,
    y0: jfloat,
    x1: jfloat,
    y1: jfloat,
    colors: jintArray,
    positions: jfloatArray,
) -> jlong {
    let colors_len = env.get_array_length(colors).unwrap_or(0) as usize;
    let positions_len = env.get_array_length(positions).unwrap_or(0) as usize;
    let mut colors_array = vec![0i32; colors_len];
    env.get_int_array_region(colors, 0, colors_array.as_mut_slice())
        .unwrap();
    let mut positions_array = vec![0f32; positions_len];
    env.get_float_array_region(positions, 0, positions_array.as_mut_slice());
    set_gradient_linear(
        canvas_native_ptr,
        x0,
        y0,
        x1,
        y1,
        colors_array.len(),
        colors_array.as_mut_ptr() as _,
        positions_array.len(),
        positions_array.as_mut_ptr() as _,
        false,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetStrokeGradientLinear(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x0: jfloat,
    y0: jfloat,
    x1: jfloat,
    y1: jfloat,
    colors: jintArray,
    positions: jfloatArray,
) -> jlong {
    let colors_len = env.get_array_length(colors).unwrap_or(0) as usize;
    let positions_len = env.get_array_length(positions).unwrap_or(0) as usize;
    let mut colors_array = vec![0i32; colors_len];
    env.get_int_array_region(colors, 0, colors_array.as_mut_slice())
        .unwrap();
    let mut positions_array = vec![0f32; positions_len];
    env.get_float_array_region(positions, 0, positions_array.as_mut_slice());
    set_gradient_linear(
        canvas_native_ptr,
        x0,
        y0,
        x1,
        y1,
        colors_array.len(),
        colors_array.as_mut_ptr() as _,
        positions_array.len(),
        positions_array.as_mut_ptr() as _,
        true,
    )
}

// set from createRadialGradient()

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetFillGradientRadial(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x0: jfloat,
    y0: jfloat,
    radius_0: jfloat,
    x1: jfloat,
    y1: jfloat,
    radius_1: jfloat,
    colors: jintArray,
    positions: jfloatArray,
) -> jlong {
    let colors_len = env.get_array_length(colors).unwrap_or(0) as usize;
    let positions_len = env.get_array_length(positions).unwrap_or(0) as usize;
    let mut colors_array = vec![0i32; colors_len];
    env.get_int_array_region(colors, 0, colors_array.as_mut_slice())
        .unwrap();
    let mut positions_array = vec![0f32; positions_len];
    env.get_float_array_region(positions, 0, positions_array.as_mut_slice());
    set_gradient_radial(
        canvas_native_ptr,
        x0,
        y0,
        radius_0,
        x1,
        y1,
        radius_1,
        colors_array.len(),
        colors_array.as_mut_ptr() as _,
        positions_array.len(),
        positions_array.as_mut_ptr() as _,
        false,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetStrokeGradientRadial(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x0: jfloat,
    y0: jfloat,
    radius_0: jfloat,
    x1: jfloat,
    y1: jfloat,
    radius_1: jfloat,
    colors: jintArray,
    positions: jfloatArray,
) -> jlong {
    let colors_len = env.get_array_length(colors).unwrap_or(0) as usize;
    let positions_len = env.get_array_length(positions).unwrap_or(0) as usize;
    let mut colors_array = vec![0i32; colors_len];
    env.get_int_array_region(colors, 0, colors_array.as_mut_slice())
        .unwrap();
    let mut positions_array = vec![0f32; positions_len];
    env.get_float_array_region(positions, 0, positions_array.as_mut_slice());
    set_gradient_radial(
        canvas_native_ptr,
        x0,
        y0,
        radius_0,
        x1,
        y1,
        radius_1,
        colors_array.len(),
        colors_array.as_mut_ptr() as _,
        positions_array.len(),
        positions_array.as_mut_ptr() as _,
        true,
    )
}

// drawImage()


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeDrawImageRaw(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    _image: jbyteArray,
    original_width: jint,
    original_height: jint,
    dx: jfloat,
    dy: jfloat,
) -> jlong {
    let length = env.get_array_length(_image).unwrap_or(0);
    let mut pixels_to_draw = vec![0i8; length as usize];
    env.get_byte_array_region(_image, 0, pixels_to_draw.as_mut_slice());
    let mut buf = to_byte_slice(pixels_to_draw.as_mut_slice());
    let image_pixels_ptr = buf.as_mut_ptr();
    let ptr = draw_image(
        canvas_native_ptr,
        image_pixels_ptr,
        pixels_to_draw.len(),
        original_width,
        original_height,
        dx,
        dy,
    );
    return ptr;
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeDrawImage(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    _image: JObject,
    dx: jfloat,
    dy: jfloat,
) -> jlong {
    let native_interface = env.get_native_interface();
    let bitmap_to_draw = _image.into_inner();
    let bitmapInfo_to_draw = Box::into_raw(Box::new(AndroidBitmapInfo::default()));

    if AndroidBitmap_getInfo(native_interface, bitmap_to_draw, bitmapInfo_to_draw)
        < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("Get Bitmap Info Failed");
        return 0;
    }
    let info_to_draw = Box::from_raw(bitmapInfo_to_draw);
    let image_info_to_draw = ImageInfo::new_n32_premul(
        ISize::new(info_to_draw.width as i32, info_to_draw.height as i32),
        None,
    );
    let mut _dstPixelsToDraw = null_mut() as *mut c_void;
    let dstPixelsToDraw: *mut *mut c_void = &mut _dstPixelsToDraw;
    if AndroidBitmap_lockPixels(native_interface, bitmap_to_draw, dstPixelsToDraw)
        < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("Get Bitmap Lock Failed");
        return 0;
    }
    let ratio_to_draw = mem::size_of_val(&dstPixelsToDraw) / mem::size_of::<u8>();
    let length_to_draw =
        ((info_to_draw.width * info_to_draw.height) * ratio_to_draw as u32) as usize;
    let new_len_to_draw = &length_to_draw * ratio_to_draw;
    let new_cap_to_draw = &length_to_draw * ratio_to_draw;
    let ptr_to_draw = _dstPixelsToDraw as *mut _;
    let pixels_to_draw: &mut [u8] =
        std::slice::from_raw_parts_mut(ptr_to_draw, length_to_draw as usize);

    let image_pixels_ptr = pixels_to_draw.as_mut_ptr();
    let ptr = draw_image(
        canvas_native_ptr,
        image_pixels_ptr,
        pixels_to_draw.len(),
        info_to_draw.width as _,
        info_to_draw.height as _,
        dx,
        dy,
    );
    if AndroidBitmap_unlockPixels(native_interface, bitmap_to_draw) < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("Unlock Bitmap Failed");
    }
    return ptr;
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeDrawImageDwRaw(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    _image: jbyteArray,
    original_width: jint,
    original_height: jint,
    dx: jfloat,
    dy: jfloat,
    d_width: jfloat,
    d_height: jfloat,
) -> jlong {
    let length = env.get_array_length(_image).unwrap_or(0);
    let mut pixels_to_draw = vec![0i8; length as usize];
    env.get_byte_array_region(_image, 0, pixels_to_draw.as_mut_slice());
    let mut buf = to_byte_slice(pixels_to_draw.as_mut_slice());
    let image_pixels_ptr = buf.as_mut_ptr();
    draw_image_dw(
        canvas_native_ptr,
        image_pixels_ptr,
        pixels_to_draw.len(),
        original_width,
        original_height,
        dx,
        dy,
        d_width,
        d_height,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeDrawImageDw(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    _image: JObject,
    dx: jfloat,
    dy: jfloat,
    d_width: jfloat,
    d_height: jfloat,
) -> jlong {
    let native_interface = env.get_native_interface();
    let bitmap_to_draw = _image.into_inner();
    let bitmapInfo_to_draw = Box::into_raw(Box::new(AndroidBitmapInfo::default()));

    if AndroidBitmap_getInfo(native_interface, bitmap_to_draw, bitmapInfo_to_draw)
        < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("Get Bitmap Info Failed Dw");
        return 0;
    }
    let info_to_draw = Box::from_raw(bitmapInfo_to_draw);
    let image_info_to_draw = ImageInfo::new_n32_premul(
        ISize::new(info_to_draw.width as i32, info_to_draw.height as i32),
        None,
    );
    let mut _dstPixelsToDraw = null_mut() as *mut c_void;
    let dstPixelsToDraw: *mut *mut c_void = &mut _dstPixelsToDraw;
    if AndroidBitmap_lockPixels(native_interface, bitmap_to_draw, dstPixelsToDraw)
        < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("Get Bitmap Lock Failed Dw");
        return 0;
    }
    let ratio_to_draw = mem::size_of_val(&dstPixelsToDraw) / mem::size_of::<u8>();
    let length_to_draw =
        ((info_to_draw.width * info_to_draw.height) * ratio_to_draw as u32) as usize;
    let new_len_to_draw = &length_to_draw * ratio_to_draw;
    let new_cap_to_draw = &length_to_draw * ratio_to_draw;
    let ptr_to_draw = _dstPixelsToDraw as *mut _;
    let pixels_to_draw: &mut [u8] =
        std::slice::from_raw_parts_mut(ptr_to_draw, length_to_draw as usize);

    let image_pixels_ptr = pixels_to_draw.as_mut_ptr();
    let ptr = draw_image_dw(
        canvas_native_ptr,
        image_pixels_ptr,
        pixels_to_draw.len(),
        info_to_draw.width as _,
        info_to_draw.height as _,
        dx,
        dy,
        d_width,
        d_height,
    );
    AndroidBitmap_unlockPixels(native_interface, bitmap_to_draw);
    return ptr;
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeDrawImageSwRaw(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    _image: jbyteArray,
    original_width: jint,
    original_height: jint,
    sx: jfloat,
    sy: jfloat,
    s_width: jfloat,
    s_height: jfloat,
    dx: jfloat,
    dy: jfloat,
    d_width: jfloat,
    d_height: jfloat,
) -> jlong {
    let length = env.get_array_length(_image).unwrap_or(0);
    let mut pixels_to_draw = vec![0i8; length as usize];
    env.get_byte_array_region(_image, 0, pixels_to_draw.as_mut_slice());
    let mut buf = to_byte_slice(pixels_to_draw.as_mut_slice());
    let image_pixels_ptr = buf.as_mut_ptr();

    draw_image_sw(
        canvas_native_ptr,
        image_pixels_ptr,
        pixels_to_draw.len(),
        original_width,
        original_height,
        sx,
        sy,
        s_width,
        s_height,
        dx,
        dy,
        d_width,
        d_height,
    )
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeDrawImageSw(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    _image: JObject,
    sx: jfloat,
    sy: jfloat,
    s_width: jfloat,
    s_height: jfloat,
    dx: jfloat,
    dy: jfloat,
    d_width: jfloat,
    d_height: jfloat,
) -> jlong {
    let native_interface = env.get_native_interface();
    let bitmap_to_draw = _image.into_inner();
    let bitmapInfo_to_draw = Box::into_raw(Box::new(AndroidBitmapInfo::default()));

    if AndroidBitmap_getInfo(native_interface, bitmap_to_draw, bitmapInfo_to_draw)
        < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("Get Bitmap Info Failed Sw");
        return 0;
    }
    let info_to_draw = Box::from_raw(bitmapInfo_to_draw);
    let image_info_to_draw = ImageInfo::new_n32_premul(
        ISize::new(info_to_draw.width as i32, info_to_draw.height as i32),
        None,
    );
    let mut _dstPixelsToDraw = null_mut() as *mut c_void;
    let dstPixelsToDraw: *mut *mut c_void = &mut _dstPixelsToDraw;
    if AndroidBitmap_lockPixels(native_interface, bitmap_to_draw, dstPixelsToDraw)
        < ANDROID_BITMAP_RESULT_SUCCESS
    {
        debug!("Get Bitmap Lock Failed Sw");
        return 0;
    }
    let ratio_to_draw = mem::size_of_val(&dstPixelsToDraw) / mem::size_of::<u8>();
    let length_to_draw =
        ((info_to_draw.width * info_to_draw.height) * ratio_to_draw as u32) as usize;
    let new_len_to_draw = &length_to_draw * ratio_to_draw;
    let new_cap_to_draw = &length_to_draw * ratio_to_draw;
    let ptr_to_draw = _dstPixelsToDraw as *mut _;
    let pixels_to_draw: &mut [u8] =
        std::slice::from_raw_parts_mut(ptr_to_draw, length_to_draw as usize);
    let image_pixels_ptr = pixels_to_draw.as_mut_ptr();
    let ptr = draw_image_sw(
        canvas_native_ptr,
        image_pixels_ptr,
        pixels_to_draw.len(),
        info_to_draw.width as _,
        info_to_draw.height as _,
        sx,
        sy,
        s_width,
        s_height,
        dx,
        dy,
        d_width,
        d_height,
    );
    AndroidBitmap_unlockPixels(native_interface, bitmap_to_draw);

    return ptr;
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeEllipse(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    radius_x: jfloat,
    radius_y: jfloat,
    rotation: jfloat,
    start_angle: jfloat,
    end_angle: jfloat,
    anticlockwise: jboolean,
) -> jlong {
    ellipse(
        canvas_native_ptr,
        true,
        x,
        y,
        radius_x,
        radius_y,
        rotation,
        start_angle,
        end_angle,
        anticlockwise == JNI_TRUE,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeFillPathRule(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    path: jlong,
    rule: JString,
) -> jlong {
    fill_path_rule(canvas_native_ptr, path, env.get_string(rule).unwrap().as_ptr() as _)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeFillRule(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    rule: JString,
) -> jlong {
    fill_rule(canvas_native_ptr, env.get_string(rule).unwrap().as_ptr() as _)
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeFill(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jlong {
    fill(canvas_native_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeFillRect(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jlong {
    drawRect(env, canvas_native_ptr, x, y, width, height, false)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeFillText(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    text: JString,
    x: jfloat,
    y: jfloat,
    width: jfloat,
) -> jlong {
    drawText(env, canvas_native_ptr, text, x, y, width, false)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeLineTo(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
) -> jlong {
    line_to(canvas_native_ptr, true, x, y)
}

static CANVAS_TEXT_METRICS: &str = "com/github/triniwiz/canvas/CanvasTextMetrics";

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeMeasureText(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    text: JString,
) -> jobject {
    let mut object = env.new_object(CANVAS_TEXT_METRICS, "()V", &[]);
    let mut result = object.unwrap();
    let txt = env.get_string(text).unwrap();
    let measurement = get_measure_text(canvas_native_ptr, txt.as_ptr() as _);
    let value = JValue::from(measurement.width);
    env.set_field(result, "width", "F", value);
    result.into_inner()
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeMoveTo(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
) -> jlong {
    move_to(canvas_native_ptr, true, x, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeQuadraticCurveTo(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    cpx: jfloat,
    cpy: jfloat,
    x: jfloat,
    y: jfloat,
) -> jlong {
    quadratic_curve_to(canvas_native_ptr, true, cpx, cpy, x, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeRect(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jlong {
    rect(canvas_native_ptr, true, x, y, width, height)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeRestore(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jlong {
    restore(canvas_native_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeRotate(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    angle: jfloat,
) -> jlong {
    rotate(canvas_native_ptr, angle)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSave(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jlong {
    save(canvas_native_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeScale(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
) -> jlong {
    scale(canvas_native_ptr, x, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetLineDash(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    array: jfloatArray,
) -> jlong {
    let size = env.get_array_length(array).unwrap_or(0) as usize;
    let mut buffer = vec![0f32; size];
    env.get_float_array_region(array, 0, buffer.as_mut_slice());
    set_line_dash(canvas_native_ptr, size, buffer.as_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetLineDashOffset(env: JNIEnv,
                                                                                                          _: JClass,
                                                                                                          canvas_native_ptr: jlong,
                                                                                                          offset: jfloat, ) -> jlong {
    set_line_dash_offset(canvas_native_ptr, offset)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetTransform(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    a: jfloat,
    b: jfloat,
    c: jfloat,
    d: jfloat,
    e: jfloat,
    f: jfloat,
) -> jlong {
    set_transform(canvas_native_ptr, a, b, c, d, e, f)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeStroke(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jlong {
    stroke(canvas_native_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeStrokePath(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    path: jlong,
) -> jlong {
    stroke_path(canvas_native_ptr, path)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeStrokeRect(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jlong {
    drawRect(env, canvas_native_ptr, x, y, width, height, true)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeStrokeText(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    text: JString,
    x: jfloat,
    y: jfloat,
    width: jfloat,
) -> jlong {
    drawText(env, canvas_native_ptr, text, x, y, width, true)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeTransform(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    a: jfloat,
    b: jfloat,
    c: jfloat,
    d: jfloat,
    e: jfloat,
    f: jfloat,
) -> jlong {
    transform(canvas_native_ptr, a, b, c, d, e, f)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeTranslate(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
) -> jlong {
    translate(canvas_native_ptr, x, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetLineWidth(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    line_width: jfloat,
) -> jlong {
    set_line_width(canvas_native_ptr, line_width)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetGlobalCompositeOperation(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    composite: JString,
) -> jlong {
    set_global_composite_operation(
        canvas_native_ptr,
        env.get_string(composite).unwrap().as_ptr() as _,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetGlobalAlpha(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    alpha: u8,
) -> jlong {
    set_global_alpha(canvas_native_ptr, alpha)
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetLineCap(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    line_cap: JString,
) -> jlong {
    set_line_cap(
        canvas_native_ptr,
        env.get_string(line_cap).unwrap().as_ptr() as _,
    )
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetLineJoin(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    line_cap: JString,
) -> jlong {
    set_line_join(
        canvas_native_ptr,
        env.get_string(line_cap).unwrap().as_ptr() as _,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetShadowBlur(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    level: jfloat,
) -> jlong {
    set_shadow_blur(canvas_native_ptr, level)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetShadowColor(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    color: jint,
) -> jlong {
    set_shadow_color(canvas_native_ptr, color as u32)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetShadowOffsetX(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    x: jfloat,
) -> jlong {
    set_shadow_offset_x(canvas_native_ptr, x)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetShadowOffsetY(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    y: jfloat,
) -> jlong {
    set_shadow_offset_y(canvas_native_ptr, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetFont(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    font: JString,
) -> jlong {
    set_font(
        canvas_native_ptr,
        env.get_string(font).unwrap().as_ptr() as _,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeCreateImageData(
    env: JNIEnv,
    _: JClass,
    width: jint,
    height: jint,
) -> jbyteArray {
    let mut image_data = create_image_data(width, height);
    env.byte_array_from_slice(image_data.as_mut_slice())
        .unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativePutImageData(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    width: jint,
    height: jint,
    array: jbyteArray,
    x: jfloat,
    y: jfloat,
    dirty_x: jfloat,
    dirty_y: jfloat,
    dirty_width: jint,
    dirty_height: jint,
) -> jlong {
    let mut array_to_write = env.convert_byte_array(array).unwrap();
    let mut slice = array_to_write.as_mut_slice();
    put_image_data(
        canvas_native_ptr,
        slice.as_mut_ptr(),
        slice.len(),
        width,
        height,
        x,
        y,
        dirty_x,
        dirty_y,
        dirty_width,
        dirty_height,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeGetImageData(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    sx: jfloat,
    sy: jfloat,
    sw: size_t,
    sh: size_t,
) -> jbyteArray {
    let result = get_image_data(canvas_native_ptr, sx, sy, sw, sh);
    let empty_slice = [0u8; 0];
    let empty_array = env.byte_array_from_slice(&empty_slice).unwrap();
    env.byte_array_from_slice(result.1.as_slice())
        .unwrap_or(empty_array)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetImageSmoothingEnabled(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    enabled: jboolean,
) -> jlong {
    set_image_smoothing_enabled(canvas_native_ptr, enabled == JNI_TRUE)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetImageSmoothingQuality(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    quality: JString,
) -> jlong {
    set_image_smoothing_quality(
        canvas_native_ptr,
        env.get_string(quality).unwrap().as_ptr() as _,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetTextAlignment(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
    alignment: JString,
) -> jlong {
    let string = env.get_string(alignment);
    if string.is_ok() {
        let text_alignment = string.unwrap();
        return set_text_align(canvas_native_ptr, text_alignment.as_ptr() as _);
    }
    canvas_native_ptr
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeResetTransform(
    env: JNIEnv,
    _: JClass,
    canvas_native_ptr: jlong,
) -> jlong {
    reset_transform(canvas_native_ptr)
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeIsPointInPath(_env: JNIEnv,
                                                                                                      _: JClass, canvas_ptr: i64, x: f32, y: f32) -> jboolean {
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_ptr as *mut _) };
    let path = canvas_native.path.clone();
    let _ = Box::into_raw(canvas_native);
    let path = Box::into_raw(Box::new(path)) as i64;
    let rule = CString::new("nonzero").unwrap().into_raw();
    let result = is_point_in_path(canvas_ptr, path, x, y, rule);
    let _ = unsafe { CString::from_raw(rule) };
    let _ = Box::from_raw(path as *mut c_void);
    if result { return JNI_TRUE; }
    return JNI_FALSE;
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeIsPointInPathWithRule(env: JNIEnv,
                                                                                                              _: JClass, canvas_ptr: i64, x: f32, y: f32, fill_rule: JString) -> jboolean {
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_ptr as *mut _) };
    let path = canvas_native.path.clone();
    let _ = Box::into_raw(canvas_native);
    let path = Box::into_raw(Box::new(path)) as i64;
    let default = env.new_string("nonzero").unwrap();
    let rule = env.get_string(fill_rule).unwrap_or(JavaStr::from_env(&env, default).unwrap());
    let result = is_point_in_path(canvas_ptr, path, x, y, rule.get_raw());
    let _ = Box::from_raw(path as *mut c_void);
    if result { return JNI_TRUE; }
    return JNI_FALSE;
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeIsPointInPathWithPathRule(env: JNIEnv,
                                                                                                                  _: JClass, canvas_ptr: i64, path: jlong, x: f32, y: f32, fill_rule: JString) -> jboolean {
    let default = env.new_string("nonzero").unwrap();
    let rule = env.get_string(fill_rule).unwrap_or(JavaStr::from_env(&env, default).unwrap());
    let result = is_point_in_path(canvas_ptr, path, x, y, rule.get_raw());
    if result { return JNI_TRUE; }
    return JNI_FALSE;
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeIsPointInStroke(_env: JNIEnv,
                                                                                                        _: JClass, canvas_ptr: i64, x: f32, y: f32) -> jboolean {
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_ptr as *mut _) };
    let path = canvas_native.path.clone();
    let _ = Box::into_raw(canvas_native);
    let path = Box::into_raw(Box::new(path)) as i64;
    let result = is_point_in_stroke(canvas_ptr, path, x, y);
    let _ = Box::from_raw(path as *mut c_void);
    if result { return JNI_TRUE; }
    return JNI_FALSE;
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeIsPointInStrokeWithPath(_env: JNIEnv,
                                                                                                                _: JClass, canvas_ptr: i64, path: jlong, x: f32, y: f32) -> jboolean {
    let result = is_point_in_stroke(canvas_ptr, path, x, y);
    if result { return JNI_TRUE; }
    return JNI_FALSE;
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeFreePath(
    _env: JNIEnv,
    _: JClass,
    path: jlong,
) {
    free_path_2d(path)
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeInit(
    env: JNIEnv,
    _: JClass,
) -> jlong {
    Box::into_raw(Box::new(Path::new())) as *mut _ as i64
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeInitWithPath(
    env: JNIEnv,
    _: JClass,
    path_ptr: jlong,
) -> jlong {
    create_path_from_path(path_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeInitWithData(
    env: JNIEnv,
    _: JClass,
    data: JString,
) -> jlong {
    create_path_2d_from_path_data(env.get_string(data).unwrap().as_ptr() as _)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeAddPath(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
    path_to_add_ptr: jlong,
    matrix: jlong,
) -> jlong {
    add_path_to_path_with_matrix(path_native_ptr, path_to_add_ptr, matrix)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeClosePath(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
) -> jlong {
    close_path(path_native_ptr, false)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeMoveTo(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
) -> jlong {
    move_to(path_native_ptr, false, x, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeLineTo(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
) -> jlong {
    line_to(path_native_ptr, false, x, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeBezierCurveTo(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
    cp1x: jfloat,
    cp1y: jfloat,
    cp2x: jfloat,
    cp2y: jfloat,
    x: jfloat,
    y: jfloat,
) -> jlong {
    bezier_curve_to(path_native_ptr, false, cp1x, cp1y, cp2x, cp2y, x, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeQuadraticCurveTo(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
    cpx: jfloat,
    cpy: jfloat,
    x: jfloat,
    y: jfloat,
) -> jlong {
    quadratic_curve_to(path_native_ptr, false, cpx, cpy, x, y)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeArc(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    radius: jfloat,
    start_angle: jfloat,
    end_angle: jfloat,
    anticlockwise: jboolean,
) -> jlong {
    arc(
        path_native_ptr,
        false,
        x,
        y,
        radius,
        start_angle,
        end_angle,
        anticlockwise == JNI_TRUE,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeEllipse(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    radius_x: jfloat,
    radius_y: jfloat,
    rotation: jfloat,
    start_angle: jfloat,
    end_angle: jfloat,
    anticlockwise: jboolean,
) -> jlong {
    ellipse(
        path_native_ptr,
        false,
        x,
        y,
        radius_x,
        radius_y,
        rotation,
        start_angle,
        end_angle,
        anticlockwise == JNI_TRUE,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeArcTo(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
    x1: jfloat,
    y1: jfloat,
    x2: jfloat,
    y2: jfloat,
    radius: jfloat,
) -> jlong {
    arc_to(path_native_ptr, false, x1, y1, x2, y2, radius)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasPath2D_nativeRect(
    env: JNIEnv,
    _: JClass,
    path_native_ptr: jlong,
    x: jfloat,
    y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jlong {
    rect(path_native_ptr, false, x, y, width, height)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeSetCurrentTransform(env: JNIEnv,
                                                                                                            _: JClass, canvas_ptr: jlong, matrix: jlong) -> jlong {
    set_current_transform(canvas_ptr, matrix)
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasRenderingContext2D_nativeGetCurrentTransform(env: JNIEnv,
                                                                                                            _: JClass, canvas_ptr: jlong) -> jlong {
    get_current_transform(canvas_ptr)
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasDOMMatrix_nativeInit(env: JNIEnv,
                                                                                    _: JClass, ) -> jlong {
    create_matrix()
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasDOMMatrix_nativeFreeMatrix(env: JNIEnv,
                                                                                          _: JClass, matrix: jlong) {
    free_matrix(matrix)
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasDOMMatrix_nativeSetMatrix(env: JNIEnv,
                                                                                         _: JClass, matrix: jlong, matrix_data: jfloatArray) -> jlong {
    let length = env.get_array_length(matrix_data).unwrap_or(0);
    let mut buffer = vec![0f32; length as usize];
    let _ = env.get_float_array_region(matrix_data, 0, buffer.as_mut_slice()).unwrap();
    set_matrix(matrix, buffer.as_mut_ptr() as *const c_void, buffer.len())
}


#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_CanvasDOMMatrix_nativeGetMatrix(env: JNIEnv,
                                                                                         _: JClass, matrix: jlong) -> jfloatArray {
    let mut data = get_matrix(matrix);
    let mut array = env.new_float_array(data.len() as i32).unwrap();
    env.set_float_array_region(array, 0, data.as_slice());
    array
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_SVGView_nativeInit(
    env: JNIEnv,
    _: JClass,
    svg_canvas_native_ptr: jlong,
    _bitmap: JObject,
) -> jlong {
    if svg_canvas_native_ptr > 0 {
        return svg_canvas_native_ptr;
    }

    let native_interface = env.get_native_interface();
    let bitmap = _bitmap.into_inner();

    let bitmapInfo = Box::into_raw(Box::new(AndroidBitmapInfo::default()));
    let get_info_success = AndroidBitmap_getInfo(native_interface, bitmap, bitmapInfo);
    if get_info_success < ANDROID_BITMAP_RESULT_SUCCESS {
        debug!("Get Bitmap Info Failed");
        return 0;
    }
    let info = Box::from_raw(bitmapInfo);
    let image_info =
        ImageInfo::new_n32_premul(ISize::new(info.width as i32, info.height as i32), None);
    let mut _dstPixels = null_mut() as *mut c_void;
    let dstPixels: *mut *mut c_void = &mut _dstPixels;
    let _get_lock_success = AndroidBitmap_lockPixels(native_interface, bitmap, dstPixels);
    if _get_lock_success < ANDROID_BITMAP_RESULT_SUCCESS {
        debug!("Get Bitmap Lock Failed");
        return 0;
    }
    let ratio = mem::size_of_val(&dstPixels) / mem::size_of::<u8>();
    let length = ((info.width * info.height) * ratio as u32) as usize;
    let new_len = &length * ratio;
    let new_cap = &length * ratio;
    let ptr = _dstPixels as *mut _;
    let pixels: &mut [u8] = std::slice::from_raw_parts_mut(ptr, length as usize);

    let surface_holder =
        Surface::new_raster_direct(&image_info, pixels, Some(info.stride as usize), None);
    if surface_holder.is_none() {
        return 0;
    }
    let mut surface = surface_holder.unwrap().deref().to_owned();
    let mut native_svg_canvas = SVGCanvasNative {
        surface,
        context: None,
    };

    let ptr = Box::into_raw(Box::new(native_svg_canvas)) as *mut _ as i64;

    AndroidBitmap_unlockPixels(native_interface, bitmap);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_github_triniwiz_canvas_SVGView_drawSVG(
    env: JNIEnv,
    _: JClass,
    svg_canvas_native_ptr: jlong,
    _bitmap: JObject,
    svg: JString,
) -> jlong {
    let native_interface = env.get_native_interface();
    let bitmap = _bitmap.into_inner();

    let bitmapInfo = Box::into_raw(Box::new(AndroidBitmapInfo::default()));
    let get_info_success = AndroidBitmap_getInfo(native_interface, bitmap, bitmapInfo);
    if get_info_success < ANDROID_BITMAP_RESULT_SUCCESS {
        debug!("Get Bitmap Info Failed");
        return 0;
    }
    let info = Box::from_raw(bitmapInfo);
    let image_info =
        ImageInfo::new_n32_premul(ISize::new(info.width as i32, info.height as i32), None);
    let mut _dstPixels = null_mut() as *mut c_void;
    let dstPixels: *mut *mut c_void = &mut _dstPixels;
    let _get_lock_success = AndroidBitmap_lockPixels(native_interface, bitmap, dstPixels);
    if _get_lock_success < ANDROID_BITMAP_RESULT_SUCCESS {
        debug!("Get Bitmap Lock Failed");
        return 0;
    }
    let ratio = mem::size_of_val(&dstPixels) / mem::size_of::<u8>();
    let length = ((info.width * info.height) * ratio as u32) as usize;
    let new_len = &length * ratio;
    let new_cap = &length * ratio;
    let ptr = _dstPixels as *mut _;
    let pixels: &mut [u8] = std::slice::from_raw_parts_mut(ptr, length as usize);

    let surface_holder =
        Surface::new_raster_direct(&image_info, pixels, Some(info.stride as usize), None);
    if surface_holder.is_none() {
        return svg_canvas_native_ptr;
    }

    let mut svg_canvas_native: Box<SVGCanvasNative> =
        unsafe { Box::from_raw(svg_canvas_native_ptr as *mut _) };
    let surface = surface_holder.unwrap();
    svg_canvas_native.surface = surface.deref().to_owned();

    let svg_canvas_native_ptr = Box::into_raw(svg_canvas_native) as *mut _ as i64;
    let ptr = draw_svg_image(
        svg_canvas_native_ptr,
        env.get_string(svg).unwrap().as_ptr() as _,
    );
    AndroidBitmap_unlockPixels(native_interface, bitmap);
    return ptr;
}
