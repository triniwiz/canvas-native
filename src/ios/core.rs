extern crate libc;

use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_uchar, c_uint};
use std::ptr::{null, null_mut};

use cocoa::foundation::NSAutoreleasePool;
use libc::{c_float, c_int, c_longlong, size_t};
use skia_safe::{
    Color, ColorType, Font, FontStyle, Paint, Path, PixelGeometry, Point, Surface, SurfaceProps,
    SurfacePropsFlags, Typeface,
};
use skia_safe::gpu;
use skia_safe::gpu::{BackendRenderTarget, Context};
use skia_safe::paint::{Cap, Join, Style};

use crate::common::{add_path_to_path, add_path_to_path_with_matrix, adjust_end_angle, arc, arc_to, begin_path, bezier_curve_to, CanvasArray, CanvasCompositeOperationType, CanvasNative, CanvasState, CanvasStateItem, CanvasTextMetrics, clear_canvas, clear_rect, clip, clip_path_rule, clip_rule, close_path, COLOR_BLACK, COLOR_TRANSPARENT, create_image_data, create_matrix, create_path_2d, create_path_2d_from_path_data, create_path_from_path, create_pattern, create_pattern_encoded, draw_image, draw_image_dw, draw_image_dw_encoded, draw_image_encoded, draw_image_sw, draw_image_sw_encoded, draw_rect, draw_text, ellipse, fill, fill_path_rule, fill_rule, flush, free_byte_array, free_char, free_path_2d, free_pattern, free_text_decoder, free_text_encoder, get_current_transform, get_image_data, get_matrix, get_measure_text, image_asset_flip_x, image_asset_flip_x_in_place_owned, image_asset_flip_y, image_asset_flip_y_in_place_owned, image_asset_free_bytes, image_asset_get_bytes, image_asset_get_error, image_asset_height, image_asset_load_from_path, image_asset_load_from_raw, image_asset_release, image_asset_save_path, image_asset_scale, image_asset_width, is_font_size, is_font_weight, is_point_in_path, is_point_in_stroke, line_to, move_to, NativeByteArray, NativeImageAsset, put_image_data, quadratic_curve_to, rect, reset_transform, restore, rotate, save, scale, set_current_transform, set_fill_color, set_fill_color_rgba, set_fill_pattern, set_font, set_global_alpha, set_global_composite_operation, set_gradient_linear, set_gradient_radial, set_image_smoothing_enabled, set_image_smoothing_quality, set_line_cap, set_line_dash, set_line_dash_offset, set_line_join, set_line_width, set_matrix, set_miter_limit, set_pattern_transform, set_shadow_blur, set_shadow_color, set_shadow_offset_x, set_shadow_offset_y, set_stroke_color, set_stroke_color_rgba, set_stroke_pattern, set_text_align, set_transform, snapshot_canvas, stroke, stroke_path, text_decoder_decode, text_decoder_get_encoding, text_encoder_encode, text_encoder_get_encoding, TextDecoder, TextEncoder, to_data_url, transform, translate, SurfaceKind};

struct AutoreleasePool(*mut objc::runtime::Object);

impl AutoreleasePool {
    fn new() -> Self {
        Self(unsafe { NSAutoreleasePool::new(cocoa::base::nil) })
    }
}

impl Drop for AutoreleasePool {
    fn drop(&mut self) {
        #[allow(clippy::let_unit_value)]
            unsafe {
            // the unit value here is needed  to type the return of msg_send().
            let () = msg_send![self.0, release];
        }
    }
}

#[no_mangle]
pub extern "C" fn native_free_char(text: *const c_char) {
    free_char(text)
}


#[no_mangle]
pub extern "C" fn native_free_byte_array(array: NativeByteArray) {
    free_byte_array(array)
}


#[no_mangle]
pub extern "C" fn native_destroy(canvas_ptr: c_longlong) {
    let _auto_release_pool = AutoreleasePool::new();
    let mut canvas: CanvasNative = unsafe { *Box::from_raw(canvas_ptr as *mut _) };
    let mut ctx = canvas.context.unwrap();
    ctx.abandon();
}

#[no_mangle]
pub extern "C" fn native_flush(canvas_ptr: c_longlong) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    flush(canvas_ptr)
}

#[no_mangle]
pub extern "C" fn native_to_data_url(
    canvas_ptr: c_longlong,
    format: *const c_char,
    quality: f32,
) -> *mut c_char {
    let _auto_release_pool = AutoreleasePool::new();
    to_data_url(canvas_ptr, format, ((quality * 100 as f32) as i32))
}

#[no_mangle]
pub extern "C" fn native_create_matrix() -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    create_matrix()
}

#[no_mangle]
pub extern "C" fn native_set_matrix(
    matrix: c_longlong,
    array: *const c_void,
    length: size_t,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_matrix(matrix, array, length)
}

#[no_mangle]
pub extern "C" fn native_get_matrix(matrix: c_longlong) -> CanvasArray {
    let _auto_release_pool = AutoreleasePool::new();
    let mut data = get_matrix(matrix);
    let ptr = data.as_ptr();
    let size = data.len();
    Box::into_raw(Box::new(data));
    CanvasArray {
        array: ptr as *const c_void,
        length: size,
    }
}

#[no_mangle]
pub extern "C" fn native_free_matrix_data(data: CanvasArray) {
    let _auto_release_pool = AutoreleasePool::new();
    let slice =
        unsafe { std::slice::from_raw_parts(data.array as *const _ as *const u8, data.length) };
    slice.to_vec();
}

/* */

#[no_mangle]
pub extern "C" fn native_create_path_2d() -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    create_path_2d()
}

#[no_mangle]
pub extern "C" fn native_create_path_from_path(path: c_longlong) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    create_path_from_path(path)
}

#[no_mangle]
pub extern "C" fn native_create_path_2d_from_path_data(data: *const c_char) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    create_path_2d_from_path_data(data)
}

#[no_mangle]
pub extern "C" fn native_free_path_2d(path: c_longlong) {
    let _auto_release_pool = AutoreleasePool::new();
    free_path_2d(path)
}

#[no_mangle]
pub extern "C" fn native_path_2d_add_path(
    path: c_longlong,
    path_to_add: c_longlong,
    matrix: c_longlong,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    add_path_to_path_with_matrix(path, path_to_add, matrix)
}

#[no_mangle]
pub extern "C" fn native_path_2d_close_path(path: c_longlong) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    close_path(path, false)
}

#[no_mangle]
pub extern "C" fn native_path_2d_move_to(path: c_longlong, x: c_float, y: c_float) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    move_to(path, false, x, y)
}

#[no_mangle]
pub extern "C" fn native_path_2d_line_to(path: c_longlong, x: c_float, y: c_float) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    line_to(path, false, x, y)
}

#[no_mangle]
pub extern "C" fn native_path_2d_bezier_curve_to(
    path: c_longlong,
    cp1x: c_float,
    cp1y: c_float,
    cp2x: c_float,
    cp2y: c_float,
    x: c_float,
    y: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    bezier_curve_to(path, false, cp1x, cp1y, cp2x, cp2y, x, y)
}

#[no_mangle]
pub extern "C" fn native_path_2d_quadratic_curve_to(
    path: c_longlong,
    cpx: c_float,
    cpy: c_float,
    x: c_float,
    y: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    quadratic_curve_to(path, false, cpx, cpy, x, y)
}

#[no_mangle]
pub extern "C" fn native_path_2d_arc(
    path: c_longlong,
    x: c_float,
    y: c_float,
    radius: c_float,
    start_angle: c_float,
    end_angle: c_float,
    anticlockwise: bool,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    arc(
        path,
        false,
        x,
        y,
        radius,
        start_angle,
        end_angle,
        anticlockwise,
    )
}

#[no_mangle]
pub extern "C" fn native_path_2d_arc_to(
    path: c_longlong,
    x1: c_float,
    y1: c_float,
    x2: c_float,
    y2: c_float,
    radius: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    arc_to(path, false, x1, y1, x2, y2, radius)
}

#[no_mangle]
pub extern "C" fn native_path_2d_ellipse(
    path: c_longlong,
    x: c_float,
    y: c_float,
    radius_x: c_float,
    radius_y: c_float,
    rotation: c_float,
    start_angle: c_float,
    end_angle: c_float,
    anticlockwise: bool,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    ellipse(
        path,
        false,
        x,
        y,
        radius_x,
        radius_y,
        rotation,
        start_angle,
        end_angle,
        anticlockwise,
    )
}

#[no_mangle]
pub extern "C" fn native_path_2d_rect(
    path: c_longlong,
    x: c_float,
    y: c_float,
    width: c_float,
    height: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    rect(path, false, x, y, width, height)
}

#[no_mangle]
pub extern "C" fn native_init_legacy(
    width: c_int,
    height: c_int,
    buffer_id: c_int,
    scale: c_float,
    direction: *const c_char,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let interface = gpu::gl::Interface::new_native();
    let context = Context::new_gl(interface);
    let mut ctx = context.unwrap();
    let max_bytes = width * height * 12 * 4 * 10;
    ctx.set_resource_cache_limit(max_bytes as usize);
    let mut frame_buffer = gpu::gl::FramebufferInfo::from_fboid(buffer_id as u32);
    frame_buffer.format = 0x8058; //GR_GL_RGBA8 (https://github.com/google/skia/blob/master/src/gpu/gl/GrGLDefines.h#L511)
    let target =
        BackendRenderTarget::new_gl((width as i32, height as i32), Some(1), 8, frame_buffer);
    let surface_props = SurfaceProps::new(SurfacePropsFlags::default(), PixelGeometry::Unknown);
    let surface_holder = Surface::from_backend_render_target(
        &mut ctx,
        &target,
        gpu::SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        Some(&surface_props),
    );
    let mut surface = surface_holder.unwrap();
    let mut stroke_paint = Paint::default();
    stroke_paint.set_anti_alias(false);
    stroke_paint.set_color(Color::BLACK);
    stroke_paint.set_stroke_width(1.0);
    stroke_paint.set_style(Style::Stroke);
    stroke_paint.set_stroke_join(Join::Miter);
    stroke_paint.set_stroke_cap(Cap::Butt);
    stroke_paint.set_stroke_miter(10.0);
    let mut fill_paint = Paint::default();
    fill_paint.set_anti_alias(false);
    fill_paint.set_color(Color::BLACK);
    fill_paint.set_style(Style::Fill);
    fill_paint.set_stroke_miter(10.0);
    fill_paint.set_stroke_join(Join::Miter);
    fill_paint.set_stroke_cap(Cap::Butt);
    // "10px sans-serif" Default
    let default_type_face =
        Typeface::from_name("sans-serif", FontStyle::normal()).unwrap_or(Typeface::default());
    let mut font = Font::from_typeface(&default_type_face, Some(10.0));
    let direction = unsafe { CStr::from_ptr(direction) }.to_str().unwrap_or("ltr");
    let mut canvas_native = CanvasNative {
        surface,
        stroke_paint,
        fill_paint,
        path: Path::new(),
        context: Some(ctx),
        font,
        state: vec![],
        line_dash_offset: 0.0,
        shadow_blur: 0.0,
        shadow_color: COLOR_TRANSPARENT as u32,
        shadow_offset_x: 0.0,
        shadow_offset_y: 0.0,
        image_smoothing_enabled: true,
        image_smoothing_quality: "low".to_string(),
        device_scale: scale,
        text_align: "start".to_string(),
        ios: 0,
        global_composite_operation: CanvasCompositeOperationType::SourceOver,
        line_cap: "butt".to_string(),
        line_join: "miter".to_string(),
        direction: direction.to_string(),
        miter_limit: 10.0,
        surface_kind: SurfaceKind::GPU
    };
    Box::into_raw(Box::new(canvas_native)) as *mut _ as i64
}

#[no_mangle]
pub extern "C" fn native_init(
    device: *mut c_void,
    queue: *mut c_void,
    view: *mut c_void,
    scale: c_float,
    direction: *const c_char,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let mut context = unsafe { gpu::Context::new_metal(device, queue) }.unwrap();
    let surface_props = SurfaceProps::new(SurfacePropsFlags::default(), PixelGeometry::Unknown);
    let mut surface_holder = Surface::from_ca_mtk_view(
        &mut context,
        view,
        gpu::SurfaceOrigin::TopLeft,
        Some(1),
        ColorType::BGRA8888,
        None,
        Some(&surface_props),
    );

    let mut surface = surface_holder.unwrap();
    let mut stroke_paint = Paint::default();
    stroke_paint.set_anti_alias(false);
    stroke_paint.set_color(Color::BLACK);
    stroke_paint.set_stroke_width(1.0);
    stroke_paint.set_style(Style::Stroke);
    stroke_paint.set_stroke_join(Join::Miter);
    stroke_paint.set_stroke_cap(Cap::Butt);
    stroke_paint.set_stroke_miter(10.0);
    let mut fill_paint = Paint::default();
    fill_paint.set_anti_alias(false);
    fill_paint.set_color(Color::BLACK);
    fill_paint.set_style(Style::Fill);
    fill_paint.set_stroke_miter(10.0);
    fill_paint.set_stroke_join(Join::Miter);
    fill_paint.set_stroke_cap(Cap::Butt);
    // "10px sans-serif" Default
    let default_type_face =
        Typeface::from_name("sans-serif", FontStyle::normal()).unwrap_or(Typeface::default());
    let mut font = Font::from_typeface(&default_type_face, Some(10.0));
    let ios = Box::new((device, queue, null_mut() as *mut c_void));
    let direction = unsafe { CStr::from_ptr(direction) }.to_str().unwrap_or("ltr");
    let mut canvas_native = CanvasNative {
        surface,
        stroke_paint,
        fill_paint,
        path: Path::new(),
        context: Some(context),
        font,
        state: vec![],
        line_dash_offset: 0.0,
        shadow_blur: 0.0,
        shadow_color: COLOR_TRANSPARENT as u32,
        shadow_offset_x: 0.0,
        shadow_offset_y: 0.0,
        image_smoothing_enabled: true,
        image_smoothing_quality: "low".to_string(),
        device_scale: scale,
        text_align: "start".to_string(),
        global_composite_operation: CanvasCompositeOperationType::SourceOver,
        line_cap: "butt".to_string(),
        line_join: "miter".to_string(),
        direction: direction.to_string(),
        ios: Box::into_raw(ios) as *mut _ as i64,
        miter_limit: 10.0,
        surface_kind: SurfaceKind::GPU
    };
    Box::into_raw(Box::new(canvas_native)) as *mut _ as i64
}

#[repr(C)]
pub struct CanvasDevice {
    pub device: *const c_void,
    pub queue: *const c_void,
    pub drawable: *const c_void,
}

#[no_mangle]
pub extern "C" fn native_get_ios_device(canvas_native_ptr: c_longlong) -> CanvasDevice {
    let null_ptr = null() as *const c_void;
    if canvas_native_ptr == 0 {
        return CanvasDevice {
            device: null_ptr,
            queue: null_ptr,
            drawable: null_ptr,
        };
    }
    let mut canvas_native: Box<CanvasNative> =
        unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut ios: Box<(*const c_void, *const c_void, *const c_void)> =
        unsafe { Box::from_raw(canvas_native.ios as _) };
    let mut devices = ios.clone();
    let mut devices = *devices;
    canvas_native.ios = Box::into_raw(ios) as i64;
    Box::into_raw(canvas_native) as i64;
    CanvasDevice {
        device: devices.0,
        queue: devices.1,
        drawable: devices.2,
    }
}

#[no_mangle]
pub extern "C" fn native_surface_resized(
    _width: c_int,
    _height: c_int,
    _device: *mut c_void,
    _queue: *mut c_void,
    _scale: c_float,
    current_canvas: c_longlong,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(current_canvas as *mut _) };
    Box::into_raw(canvas_native) as *mut _ as i64
}

#[no_mangle]
pub extern "C" fn native_surface_resized_legacy(
    width: c_int,
    height: c_int,
    buffer_id: c_int,
    _scale: c_float,
    canvas_native_ptr: c_longlong,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let mut canvas_native: Box<CanvasNative> =
        unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut context = canvas_native.context.unwrap();
    let max_bytes = width * height * 12 * 4 * 10;
    context.set_resource_cache_limit(max_bytes as usize);
    let mut surface = &mut canvas_native.surface;
    context.flush();
    let snapshot = surface.image_snapshot();
    // context.set_resource_cache_limit(kGrCacheMaxByteSize);
    let mut frame_buffer = gpu::gl::FramebufferInfo::from_fboid(buffer_id as u32);
    frame_buffer.format = 0x8058; //GR_GL_RGBA8 (https://github.com/google/skia/blob/master/src/gpu/gl/GrGLDefines.h#L511)
    let target =
        BackendRenderTarget::new_gl((width as i32, height as i32), Some(1), 8, frame_buffer);
    let surface_props = SurfaceProps::new(SurfacePropsFlags::default(), PixelGeometry::Unknown);
    let surface_holder = Surface::from_backend_render_target(
        &mut context,
        &target,
        gpu::SurfaceOrigin::BottomLeft,
        ColorType::n32(),
        None,
        Some(&surface_props),
    );
    let mut surface = surface_holder.unwrap();
    let mut canvas = surface.canvas();
    canvas.draw_image(snapshot, Point::new(0f32, 0f32), None);
    //canvas.flush();
    canvas_native.context = Some(context);
    canvas_native.surface = surface;
    Box::into_raw(canvas_native) as *mut _ as i64
}

fn update_surface(canvas_native_ptr: c_longlong, view: *mut c_void) -> c_longlong {
    if view.is_null() {
        return canvas_native_ptr;
    }
    let _auto_release_pool = AutoreleasePool::new();
    let mut canvas_native: CanvasNative = unsafe { *Box::from_raw(canvas_native_ptr as *mut _) };
    let mut context = canvas_native.context.unwrap();
    let surface_props = SurfaceProps::new(SurfacePropsFlags::default(), PixelGeometry::Unknown);
    let mut surface_holder = Surface::from_ca_mtk_view(
        &mut context,
        view,
        gpu::SurfaceOrigin::TopLeft,
        Some(1),
        ColorType::BGRA8888,
        None,
        Some(&surface_props),
    );

    let _ = &mut canvas_native.surface;

    let mut surface = surface_holder.unwrap();
    canvas_native.context = Some(context);
    canvas_native.surface = surface;

    Box::into_raw(Box::new(canvas_native)) as *mut _ as i64
}

#[no_mangle]
pub extern "C" fn native_is_point_in_path(canvas_ptr: i64, x: f32, y: f32) -> c_uchar {
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_ptr as *mut _) };
    let path = canvas_native.path.clone();
    let path = Box::into_raw(Box::new(path)) as i64;
    let _ = Box::into_raw(canvas_native);
    let rule = CString::new("nonzero").unwrap().into_raw();
    let result = native_is_point_in_path_with_path_rule(canvas_ptr, path, x, y, rule);
    let _ = unsafe { CString::from_raw(rule) };
    let _ = unsafe { Box::from_raw(path as *mut c_void) };
    result
}

#[no_mangle]
pub extern "C" fn native_is_point_in_path_with_rule(
    canvas_ptr: i64,
    x: f32,
    y: f32,
    fill_rule: *const c_char,
) -> c_uchar {
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_ptr as *mut _) };
    let path = canvas_native.path.clone();
    let _ = Box::into_raw(canvas_native);
    let path = Box::into_raw(Box::new(path)) as i64;
    let result = native_is_point_in_path_with_path_rule(canvas_ptr, path, x, y, fill_rule);
    let _ = unsafe { Box::from_raw(path as *mut c_void) };
    result
}

#[no_mangle]
pub extern "C" fn native_is_point_in_path_with_path_rule(
    canvas_ptr: i64,
    path: i64,
    x: f32,
    y: f32,
    fill_rule: *const c_char,
) -> c_uchar {
    let result = is_point_in_path(canvas_ptr, path, x, y, fill_rule);
    if result {
        return 1;
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn native_is_point_in_stroke(canvas_ptr: i64, x: f32, y: f32) -> c_uchar {
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_ptr as *mut _) };
    let path = canvas_native.path.clone();
    let _ = Box::into_raw(canvas_native);
    let path = Box::into_raw(Box::new(path)) as i64;
    let result = native_is_point_in_stroke_with_path(canvas_ptr, path, x, y);
    let _ = unsafe { Box::from_raw(path as *mut c_void) };
    result
}

#[no_mangle]
pub extern "C" fn native_is_point_in_stroke_with_path(
    canvas_ptr: i64,
    path: i64,
    x: f32,
    y: f32,
) -> c_uchar {
    let result = is_point_in_stroke(canvas_ptr, path, x, y);
    if result {
        return 1;
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn native_fill_rect(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
    width: c_float,
    height: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_rect(canvas_native_ptr, x, y, width, height, false)
}

#[no_mangle]
pub extern "C" fn native_stroke_rect(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
    width: c_float,
    height: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_rect(canvas_native_ptr, x, y, width, height, true)
}

#[no_mangle]
pub extern "C" fn native_begin_path(canvas_native_ptr: c_longlong) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    begin_path(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_stroke_path(
    canvas_native_ptr: c_longlong,
    path: c_longlong,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    stroke_path(canvas_native_ptr, path)
}

#[no_mangle]
pub extern "C" fn native_stroke(canvas_native_ptr: c_longlong, view: *mut c_void) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    stroke(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_fill(canvas_native_ptr: c_longlong, view: *mut c_void) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    fill(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_fill_rule(
    canvas_native_ptr: c_longlong,
    rule: *const c_char,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    fill_rule(canvas_native_ptr, rule)
}

#[no_mangle]
pub extern "C" fn native_fill_path_rule(
    canvas_native_ptr: c_longlong,
    path_ptr: c_longlong,
    rule: *const c_char,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    fill_path_rule(canvas_native_ptr, path_ptr, rule)
}

#[no_mangle]
pub extern "C" fn native_close_path(canvas_native_ptr: c_longlong) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    close_path(canvas_native_ptr, true)
}

#[no_mangle]
pub extern "C" fn native_rect(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
    width: c_float,
    height: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    rect(canvas_native_ptr, true, x, y, width, height)
}

#[no_mangle]
pub extern "C" fn native_bezier_curve_to(
    canvas_native_ptr: c_longlong,
    cp1x: c_float,
    cp1y: c_float,
    cp2x: c_float,
    cp2y: c_float,
    x: c_float,
    y: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    bezier_curve_to(canvas_native_ptr, true, cp1x, cp1y, cp2x, cp2y, x, y)
}

#[no_mangle]
pub extern "C" fn native_line_to(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    line_to(canvas_native_ptr, true, x, y)
}

#[no_mangle]
pub extern "C" fn native_ellipse(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
    radius_x: c_float,
    radius_y: c_float,
    rotation: c_float,
    start_angle: c_float,
    end_angle: c_float,
    anticlockwise: bool,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
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
        anticlockwise,
    )
}

#[no_mangle]
pub extern "C" fn native_arc_to(
    canvas_native_ptr: c_longlong,
    x1: c_float,
    y1: c_float,
    x2: c_float,
    y2: c_float,
    radius: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    arc_to(canvas_native_ptr, true, x1, y1, x2, y2, radius)
}

#[no_mangle]
pub extern "C" fn native_set_line_width(
    canvas_native_ptr: c_longlong,
    line_width: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_line_width(canvas_native_ptr, line_width)
}

#[no_mangle]
pub extern "C" fn native_arc(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
    radius: c_float,
    start_angle: c_float,
    end_angle: c_float,
    anticlockwise: bool,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    arc(
        canvas_native_ptr,
        true,
        x,
        y,
        radius,
        start_angle,
        end_angle,
        anticlockwise,
    )
}

#[no_mangle]
pub extern "C" fn native_move_to(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    move_to(canvas_native_ptr, true, x, y)
}

#[no_mangle]
pub extern "C" fn native_create_pattern(
    image_array: *mut u8,
    image_size: size_t,
    original_width: c_int,
    original_height: c_int,
    repetition: *const c_char,
) -> c_longlong {
    create_pattern(
        image_array,
        image_size,
        original_width,
        original_height,
        repetition,
    )
}

#[no_mangle]
pub extern "C" fn native_create_pattern_encoded(
    image_array: *mut u8,
    image_size: size_t,
    repetition: *const c_char,
) -> c_longlong {
    create_pattern_encoded(
        image_array,
        image_size,
        repetition,
    )
}

#[no_mangle]
pub extern "C" fn native_free_pattern(pattern: c_longlong) {
    free_pattern(pattern)
}

#[no_mangle]
pub extern "C" fn native_set_pattern_transform(
    pattern: c_longlong,
    matrix: c_longlong,
) -> c_longlong {
    set_pattern_transform(pattern, matrix)
}

#[no_mangle]
pub extern "C" fn native_set_fill_pattern(
    canvas_native_ptr: c_longlong,
    pattern: c_longlong,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_fill_pattern(canvas_native_ptr, pattern)
}

#[no_mangle]
pub extern "C" fn native_set_stroke_pattern(
    canvas_native_ptr: c_longlong,
    pattern: c_longlong,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_stroke_pattern(canvas_native_ptr, pattern)
}

#[no_mangle]
pub extern "C" fn native_set_fill_color_rgba(
    canvas_native_ptr: c_longlong,
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_fill_color_rgba(canvas_native_ptr, red, green, blue, alpha)
}

#[no_mangle]
pub extern "C" fn native_set_fill_color(canvas_native_ptr: c_longlong, color: u32) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_fill_color(canvas_native_ptr, color as u32)
}

#[no_mangle]
pub extern "C" fn native_set_fill_gradient_radial(
    canvas_native_ptr: c_longlong,
    x0: c_float,
    y0: c_float,
    radius_0: c_float,
    x1: c_float,
    y1: c_float,
    radius_1: c_float,
    colors_size: size_t,
    colors_array: *const c_uint,
    positions_size: size_t,
    positions_array: *const c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_gradient_radial(
        canvas_native_ptr,
        x0,
        y0,
        radius_0,
        x1,
        y1,
        radius_1,
        colors_size,
        colors_array,
        positions_size,
        positions_array,
        false,
    )
}

#[no_mangle]
pub extern "C" fn native_set_stroke_gradient_radial(
    canvas_native_ptr: c_longlong,
    x0: c_float,
    y0: c_float,
    radius_0: c_float,
    x1: c_float,
    y1: c_float,
    radius_1: c_float,
    colors_size: size_t,
    colors_array: *const c_uint,
    positions_size: size_t,
    positions_array: *const c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_gradient_radial(
        canvas_native_ptr,
        x0,
        y0,
        radius_0,
        x1,
        y1,
        radius_1,
        colors_size,
        colors_array,
        positions_size,
        positions_array,
        true,
    )
}

#[no_mangle]
pub extern "C" fn native_set_fill_gradient_linear(
    canvas_native_ptr: c_longlong,
    x0: c_float,
    y0: c_float,
    x1: c_float,
    y1: c_float,
    colors_size: size_t,
    colors_array: *const c_uint,
    positions_size: size_t,
    positions_array: *const c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_gradient_linear(
        canvas_native_ptr,
        x0,
        y0,
        x1,
        y1,
        colors_size,
        colors_array,
        positions_size,
        positions_array,
        false,
    )
}

#[no_mangle]
pub extern "C" fn native_set_stroke_gradient_linear(
    canvas_native_ptr: c_longlong,
    x0: c_float,
    y0: c_float,
    x1: c_float,
    y1: c_float,
    colors_size: size_t,
    colors_array: *const c_uint,
    positions_size: size_t,
    positions_array: *const c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_gradient_linear(
        canvas_native_ptr,
        x0,
        y0,
        x1,
        y1,
        colors_size,
        colors_array,
        positions_size,
        positions_array,
        true,
    )
}

#[no_mangle]
pub extern "C" fn native_set_stroke_color_rgba(
    canvas_native_ptr: c_longlong,
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_stroke_color_rgba(canvas_native_ptr, red, green, blue, alpha)
}

#[no_mangle]
pub extern "C" fn native_set_stroke_color(canvas_native_ptr: c_longlong, color: u32) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_stroke_color(canvas_native_ptr, color)
}

#[no_mangle]
pub extern "C" fn native_clear_rect(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
    width: c_float,
    height: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    clear_rect(canvas_native_ptr, x, y, width, height)
}

#[no_mangle]
pub extern "C" fn native_clear_canvas(
    canvas_native_ptr: c_longlong,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    clear_canvas(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_set_line_dash(
    canvas_native_ptr: c_longlong,
    size: size_t,
    array: *const c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_line_dash(canvas_native_ptr, size, array)
}

#[no_mangle]
pub extern "C" fn native_set_global_composite_operation(
    canvas_native_ptr: c_longlong,
    composite: *const c_char,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_global_composite_operation(canvas_native_ptr, composite)
}

#[no_mangle]
pub extern "C" fn native_set_font(
    canvas_native_ptr: c_longlong,
    font: *const c_char,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_font(canvas_native_ptr, font)
}

#[no_mangle]
pub extern "C" fn native_fill_text(
    canvas_native_ptr: c_longlong,
    text: *const c_char,
    x: c_float,
    y: c_float,
    width: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_text(canvas_native_ptr, text, x, y, width, false)
}

#[no_mangle]
pub extern "C" fn native_stroke_text(
    canvas_native_ptr: c_longlong,
    text: *const c_char,
    x: c_float,
    y: c_float,
    width: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_text(canvas_native_ptr, text, x, y, width, true)
}

#[no_mangle]
pub extern "C" fn native_scale(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    scale(canvas_native_ptr, x, y)
}

#[no_mangle]
pub extern "C" fn native_transform(
    canvas_native_ptr: c_longlong,
    a: c_float,
    b: c_float,
    c: c_float,
    d: c_float,
    e: c_float,
    f: c_float,
    _view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    transform(canvas_native_ptr, a, b, c, d, e, f)
}

#[no_mangle]
pub extern "C" fn native_set_transform(
    canvas_native_ptr: c_longlong,
    a: c_float,
    b: c_float,
    c: c_float,
    d: c_float,
    e: c_float,
    f: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    set_transform(canvas_native_ptr, a, b, c, d, e, f)
}

#[no_mangle]
pub extern "C" fn native_rotate(
    canvas_native_ptr: c_longlong,
    angle: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    rotate(canvas_native_ptr, angle)
}

#[no_mangle]
pub extern "C" fn native_translate(
    canvas_native_ptr: c_longlong,
    x: c_float,
    y: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    translate(canvas_native_ptr, x, y)
}

#[no_mangle]
pub extern "C" fn native_quadratic_curve_to(
    canvas_native_ptr: c_longlong,
    cpx: c_float,
    cpy: c_float,
    x: c_float,
    y: c_float,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    quadratic_curve_to(canvas_native_ptr, true, cpx, cpy, x, y)
}

#[no_mangle]
pub extern "C" fn native_draw_image_raw(
    canvas_native_ptr: c_longlong,
    image_array: *const u8,
    image_size: size_t,
    original_width: c_int,
    original_height: c_int,
    dx: c_float,
    dy: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_image(
        canvas_native_ptr,
        image_array,
        image_size,
        original_width,
        original_height,
        dx,
        dy,
    )
}


#[no_mangle]
pub unsafe extern "C" fn native_snapshot_canvas(
    canvas_native_ptr: c_longlong,
) -> NativeByteArray {
    snapshot_canvas(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_draw_image(
    canvas_native_ptr: c_longlong,
    image_array: *const u8,
    image_size: size_t,
    original_width: c_int,
    original_height: c_int,
    dx: c_float,
    dy: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_image_encoded(
        canvas_native_ptr,
        image_array,
        image_size,
        original_width,
        original_height,
        dx,
        dy,
    )
}

#[no_mangle]
pub extern "C" fn native_draw_image_dw_raw(
    canvas_native_ptr: c_longlong,
    image_array: *const u8,
    image_size: size_t,
    original_width: c_int,
    original_height: c_int,
    dx: c_float,
    dy: c_float,
    d_width: c_float,
    d_height: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_image_dw(
        canvas_native_ptr,
        image_array,
        image_size,
        original_width,
        original_height,
        dx,
        dy,
        d_width,
        d_height,
    )
}

#[no_mangle]
pub extern "C" fn native_draw_image_dw(
    canvas_native_ptr: c_longlong,
    image_array: *const u8,
    image_size: size_t,
    original_width: c_int,
    original_height: c_int,
    dx: c_float,
    dy: c_float,
    d_width: c_float,
    d_height: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_image_dw_encoded(
        canvas_native_ptr,
        image_array,
        image_size,
        original_width,
        original_height,
        dx,
        dy,
        d_width,
        d_height,
    )
}

#[no_mangle]
pub extern "C" fn native_draw_image_sw_raw(
    canvas_native_ptr: c_longlong,
    image_array: *const u8,
    image_size: size_t,
    original_width: c_int,
    original_height: c_int,
    sx: c_float,
    sy: c_float,
    s_width: c_float,
    s_height: c_float,
    dx: c_float,
    dy: c_float,
    d_width: c_float,
    d_height: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_image_sw(
        canvas_native_ptr,
        image_array,
        image_size,
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
pub extern "C" fn native_draw_image_sw(
    canvas_native_ptr: c_longlong,
    image_array: *const u8,
    image_size: size_t,
    original_width: c_int,
    original_height: c_int,
    sx: c_float,
    sy: c_float,
    s_width: c_float,
    s_height: c_float,
    dx: c_float,
    dy: c_float,
    d_width: c_float,
    d_height: c_float,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    draw_image_sw_encoded(
        canvas_native_ptr,
        image_array,
        image_size,
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
pub extern "C" fn native_save(canvas_native_ptr: c_longlong) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    save(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_restore(canvas_native_ptr: c_longlong) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    restore(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_measure_text(
    canvas_native_ptr: c_longlong,
    text: *const c_char,
) -> CanvasTextMetrics {
    let _auto_release_pool = AutoreleasePool::new();
    get_measure_text(canvas_native_ptr, text)
}

#[no_mangle]
pub extern "C" fn native_set_line_cap(
    canvas_native_ptr: c_longlong,
    line_cap: *const c_char,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_line_cap(canvas_native_ptr, line_cap as *mut _)
}

#[no_mangle]
pub extern "C" fn native_set_global_alpha(canvas_native_ptr: c_longlong, alpha: u8) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_global_alpha(canvas_native_ptr, alpha)
}

#[no_mangle]
pub extern "C" fn native_image_smoothing_enabled(
    canvas_native_ptr: c_longlong,
    enabled: bool,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_image_smoothing_enabled(canvas_native_ptr, enabled)
}

#[no_mangle]
pub extern "C" fn native_image_smoothing_quality(
    canvas_native_ptr: c_longlong,
    quality: *const c_char,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_image_smoothing_quality(canvas_native_ptr, quality)
}

#[no_mangle]
pub extern "C" fn native_line_dash_offset(
    canvas_native_ptr: c_longlong,
    offset: f32,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_line_dash_offset(canvas_native_ptr, offset)
}

#[no_mangle]
pub extern "C" fn native_line_join(
    canvas_native_ptr: c_longlong,
    line_cap: *const c_char,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_line_join(canvas_native_ptr, line_cap)
}

#[no_mangle]
pub extern "C" fn native_miter_limit(canvas_native_ptr: c_longlong, limit: f32) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_miter_limit(canvas_native_ptr, limit)
}

#[no_mangle]
pub extern "C" fn native_shadow_blur(canvas_native_ptr: c_longlong, limit: f32) -> c_longlong {
    set_shadow_blur(canvas_native_ptr, limit)
}

#[no_mangle]
pub extern "C" fn native_shadow_color(canvas_native_ptr: c_longlong, color: u32) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_shadow_color(canvas_native_ptr, color)
}

#[no_mangle]
pub extern "C" fn native_shadow_offset_x(canvas_native_ptr: c_longlong, x: f32) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_shadow_offset_x(canvas_native_ptr, x)
}

#[no_mangle]
pub extern "C" fn native_shadow_offset_y(canvas_native_ptr: c_longlong, y: f32) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_shadow_offset_y(canvas_native_ptr, y)
}

#[no_mangle]
pub extern "C" fn native_text_align(
    canvas_native_ptr: c_longlong,
    alignment: *const c_char,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    set_text_align(canvas_native_ptr, alignment)
}

#[no_mangle]
pub extern "C" fn native_reset_transform(canvas_native_ptr: c_longlong) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    reset_transform(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_clip(canvas_native_ptr: c_longlong, view: *mut c_void) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    clip(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_clip_rule(
    canvas_native_ptr: c_longlong,
    fill_rule: *const c_char,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    clip_rule(canvas_native_ptr, fill_rule)
}

#[no_mangle]
pub extern "C" fn native_clip_path_rule(
    canvas_native_ptr: c_longlong,
    path: c_longlong,
    fill_rule: *const c_char,
    view: *mut c_void,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let canvas_native_ptr = update_surface(canvas_native_ptr, view);
    clip_path_rule(canvas_native_ptr, path, fill_rule)
}

#[no_mangle]
pub extern "C" fn native_create_image_data(width: size_t, height: size_t) -> CanvasArray {
    let _auto_release_pool = AutoreleasePool::new();
    let mut image_data = create_image_data(width as _, height as _);
    CanvasArray {
        array: image_data.as_ptr() as *mut c_void,
        length: image_data.len(),
    }
}

#[no_mangle]
pub extern "C" fn native_put_image_data(
    canvas_native_ptr: c_longlong,
    width: size_t,
    height: size_t,
    array: *const u8,
    array_size: size_t,
    x: c_float,
    y: c_float,
    dirty_x: c_float,
    dirty_y: c_float,
    dirty_width: size_t,
    dirty_height: size_t,
) -> c_longlong {
    let _auto_release_pool = AutoreleasePool::new();
    let mut slice = unsafe { std::slice::from_raw_parts(array, array_size) };
    put_image_data(
        canvas_native_ptr,
        slice.as_ptr(),
        slice.len(),
        width as _,
        height as _,
        x,
        y,
        dirty_x,
        dirty_y,
        dirty_width as _,
        dirty_height as _,
    )
}

#[no_mangle]
pub extern "C" fn native_get_image_data(
    canvas_native_ptr: c_longlong,
    sx: c_float,
    sy: c_float,
    sw: size_t,
    sh: size_t,
) -> CanvasArray {
    let _auto_release_pool = AutoreleasePool::new();
    let mut image_data = get_image_data(canvas_native_ptr, sx, sy, sw, sh);
    CanvasArray {
        array: image_data.1.as_ptr() as *mut c_void,
        length: image_data.1.len(),
    }
}

#[no_mangle]
pub extern "C" fn native_drop_image_data(data: CanvasArray) {
    let _auto_release_pool = AutoreleasePool::new();
    let slice =
        unsafe { std::slice::from_raw_parts(data.array as *const _ as *const u8, data.length) };
    slice.to_vec();
}

#[no_mangle]
pub extern "C" fn native_drop_text_metrics(data: CanvasTextMetrics) {
    let _auto_release_pool = AutoreleasePool::new();
    Box::new(data);
}

#[no_mangle]
pub extern "C" fn native_get_current_transform(canvas_native_ptr: c_longlong) -> c_longlong {
    get_current_transform(canvas_native_ptr)
}

#[no_mangle]
pub extern "C" fn native_set_current_transform(
    canvas_native_ptr: c_longlong,
    matrix: c_longlong,
) -> c_longlong {
    set_current_transform(canvas_native_ptr, matrix)
}
