extern crate libc;

use skia_safe::{Point, Color, Surface, SurfaceProps, PixelGeometry, ColorType, Paint, Rect, SurfacePropsFlags, FontStyle, ImageInfo, ISize, Budgeted, ColorSpace, AlphaType, Vector, Path, Matrix, BlendMode, PathEffect, Shader, TileMode, ClipOp, Font, Typeface, TextBlob, FontStyleWeight, Image, Bitmap, Size, PixelRef, Pixmap, Data, SrcRectConstraint, IPoint, EncodedImageFormat, IRect};
use skia_safe::gpu::gl;
use skia_safe::paint::{Style, Join, Cap};
use libc::{c_int, c_longlong, c_float, size_t, c_char};
use skia_safe::gpu::{Context, BackendRenderTarget, SurfaceOrigin};
use crate::common::{CanvasNative, CanvasCompositeOperationType, is_font_weight, is_font_size, CanvasState, CanvasStateItem, draw_rect, draw_text, move_to, COLOR_BLACK, set_line_width, adjust_end_angle, ellipse, begin_path, stroke, fill, close_path, rect, bezier_curve_to, line_to, arc_to, arc, set_fill_color_rgba, set_gradient_radial, set_gradient_linear, set_stroke_color_rgba, clear_rect, clear_canvas, set_line_dash, set_global_composite_operation, set_font, scale, set_transform, transform, rotate, translate, quadratic_curve_to, draw_image, draw_image_dw, draw_image_sw, save, restore, set_line_cap, set_global_alpha, set_image_smoothing_enabled, set_image_smoothing_quality, set_line_dash_offset, set_line_join, set_miter_limit, set_shadow_blur, set_shadow_color, set_shadow_offset_x, set_shadow_offset_y, set_text_align, CanvasTextMetrics, get_measure_text, reset_transform, clip_rule, create_image_data, CanvasImageData, get_image_data, put_image_data, draw_image_encoded, draw_image_dw_encoded, draw_image_sw_encoded};
use std::mem;
use std::ffi::CStr;
use skia_safe::gradient_shader::GradientShaderColors;
use std::os::raw::c_void;


#[no_mangle]
pub extern fn native_init(width: c_int, height: c_int, buffer_id: c_int, scale: c_float) -> c_longlong {
    let interface = gl::Interface::new_native();
    let context = Context::new_gl(Some(&interface.unwrap()));
    let mut ctx = context.unwrap();
    let mut frame_buffer = gl::FramebufferInfo::from_fboid(buffer_id as u32);
    frame_buffer.format = 0x8058; //GR_GL_RGBA8 (https://github.com/google/skia/blob/master/src/gpu/gl/GrGLDefines.h#L511)
    let target = BackendRenderTarget::new_gl((width as i32, height as i32), Some(0), 8, frame_buffer);
    let surface_props = SurfaceProps::new(SurfacePropsFlags::default(), PixelGeometry::Unknown);
    let surface_holder = Surface::from_backend_render_target(&mut ctx, &target, SurfaceOrigin::TopLeft, ColorType::RGBA8888, None, Some(&surface_props));
    let mut surface = surface_holder.unwrap();
    let mut stroke_paint = Paint::default();
    stroke_paint.set_anti_alias(true);
    stroke_paint.set_color(Color::from(COLOR_BLACK));
    stroke_paint.set_stroke_width(1.0);
    stroke_paint.set_style(Style::Stroke);
    stroke_paint.set_stroke_join(Join::Miter);
    stroke_paint.set_stroke_cap(Cap::Butt);
    stroke_paint.set_stroke_miter(10.0);
    let mut fill_paint = Paint::default();
    fill_paint.set_anti_alias(true);
    fill_paint.set_color(Color::from(COLOR_BLACK));
    fill_paint.set_style(Style::Fill);
    fill_paint.set_stroke_miter(10.0);
    fill_paint.set_stroke_join(Join::Miter);
    fill_paint.set_stroke_cap(Cap::Butt);
    // "10px sans-serif" Default
    let default_type_face = Typeface::from_name("sans-serif", FontStyle::normal()).unwrap_or(
        Typeface::default()
    );
    let mut font = Font::from_typeface(
        &default_type_face,
        Some(10.0),
    );
    let canvas = surface.canvas();
    canvas.scale((scale, scale));
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
        shadow_color: COLOR_BLACK as u32,
        shadow_offset_x: 0.0,
        shadow_offset_y: 0.0,
        image_smoothing_enabled: false,
        image_smoothing_quality: "low".to_string(),
        device_scale: scale,
        text_align: "left".to_string(),
    };


    Box::into_raw(Box::new(canvas_native)) as *mut _ as i64
}


#[no_mangle]
pub extern fn native_surface_resized(width: c_int, height: c_int, buffer_id: c_int, current_canvas: c_longlong) -> c_longlong {
    let mut canvas_native: Box<CanvasNative> = unsafe {
        Box::from_raw(current_canvas as *mut _)
    };
    let interface = gl::Interface::new_native();
    let mut context = Context::new_gl(Some(&interface.unwrap()));
    let mut ctx = context.unwrap();
    let mut frame_buffer = gl::FramebufferInfo::from_fboid(buffer_id as u32);
    frame_buffer.format = 0x8058; //GR_GL_RGBA8 (https://github.com/google/skia/blob/master/src/gpu/gl/GrGLDefines.h#L511)
    let target = BackendRenderTarget::new_gl((width as i32, height as i32), Some(0), 8, frame_buffer);
    let surface_props = SurfaceProps::new(SurfacePropsFlags::default(), PixelGeometry::Unknown);
    let surface_holder = Surface::from_backend_render_target(&mut ctx, &target, SurfaceOrigin::BottomLeft, ColorType::RGBA8888, None, Some(&surface_props));
    let mut surface = surface_holder.unwrap();
    let canvas = surface.canvas();
    canvas_native.surface = surface;
    canvas_native.context = Some(ctx);
    Box::into_raw(canvas_native) as *mut _ as i64
}

#[no_mangle]
pub extern fn native_fill_rect(canvas_native_ptr: c_longlong, x: c_float, y: c_float, width: c_float, height: c_float) -> c_longlong {
    draw_rect(canvas_native_ptr, x, y, width, height, false)
}

#[no_mangle]
pub extern fn native_stroke_rect(canvas_native_ptr: c_longlong, x: c_float, y: c_float, width: c_float, height: c_float) -> c_longlong {
    draw_rect(canvas_native_ptr, x, y, width, height, true)
}

#[no_mangle]
pub extern fn native_begin_path(canvas_native_ptr: c_longlong) -> c_longlong {
    begin_path(canvas_native_ptr)
}


#[no_mangle]
pub extern fn native_stroke(canvas_native_ptr: c_longlong) -> c_longlong {
    stroke(canvas_native_ptr)
}

#[no_mangle]
pub extern fn native_fill(canvas_native_ptr: c_longlong) -> c_longlong {
    fill(canvas_native_ptr)
}

#[no_mangle]
pub extern fn native_close_path(canvas_native_ptr: c_longlong) -> c_longlong {
    close_path(canvas_native_ptr, true)
}


#[no_mangle]
pub extern fn native_rect(canvas_native_ptr: c_longlong, x: c_float, y: c_float, width: c_float, height: c_float) -> c_longlong {
    rect(canvas_native_ptr, true, x, y, width, height)
}


#[no_mangle]
pub extern fn native_bezier_curve_to(canvas_native_ptr: c_longlong, cp1x: c_float, cp1y: c_float, cp2x: c_float, cp2y: c_float, x: c_float, y: c_float) -> c_longlong {
    bezier_curve_to(canvas_native_ptr, true, cp1x, cp1y, cp2x, cp2y, x, y)
}

#[no_mangle]
pub extern fn native_line_to(canvas_native_ptr: c_longlong, x: c_float, y: c_float) -> c_longlong {
    line_to(canvas_native_ptr, true, x, y)
}


#[no_mangle]
pub extern fn native_ellipse(canvas_native_ptr: c_longlong, x: c_float, y: c_float, radius_x: c_float, radius_y: c_float, rotation: c_float, start_angle: c_float, end_angle: c_float, anticlockwise: bool) -> c_longlong {
    ellipse(canvas_native_ptr, true, x, y, radius_x, radius_y, rotation, start_angle, end_angle, anticlockwise)
}


#[no_mangle]
pub extern fn native_arc_to(canvas_native_ptr: c_longlong, x1: c_float, y1: c_float, x2: c_float, y2: c_float, radius: c_float) -> c_longlong {
    arc_to(canvas_native_ptr, true, x1, y1, x2, y2, radius)
}

#[no_mangle]
pub extern fn native_set_line_width(canvas_native_ptr: c_longlong, line_width: c_float) -> c_longlong {
    set_line_width(canvas_native_ptr, line_width)
}

#[no_mangle]
pub extern fn native_arc(canvas_native_ptr: c_longlong, x: c_float, y: c_float, radius: c_float, start_angle: c_float, end_angle: c_float, anticlockwise: bool) -> c_longlong {
    arc(canvas_native_ptr, true, x, y, radius, start_angle, end_angle, anticlockwise)
}

#[no_mangle]
pub extern fn native_move_to(canvas_native_ptr: c_longlong, x: c_float, y: c_float) -> c_longlong {
    move_to(canvas_native_ptr, true, x, y)
}


#[no_mangle]
pub extern fn native_set_fill_color_rgba(canvas_native_ptr: c_longlong, red: u8, green: u8, blue: u8, alpha: u8) -> c_longlong {
    set_fill_color_rgba(canvas_native_ptr, red, green, blue, alpha)
}


#[no_mangle]
pub extern fn native_set_fill_gradient_radial(canvas_native_ptr: c_longlong, x0: c_float, y0: c_float, radius_0: c_float, x1: c_float, y1: c_float, radius_1: c_float, colors_size: size_t, colors_array: *const size_t, positions_size: size_t, positions_array: *const c_float) -> c_longlong {
    set_gradient_radial(canvas_native_ptr, x0, y0, radius_0, x1, y1, radius_1, colors_size, colors_array, positions_size, positions_array, false)
}

#[no_mangle]
pub extern fn native_set_stroke_gradient_radial(canvas_native_ptr: c_longlong, x0: c_float, y0: c_float, radius_0: c_float, x1: c_float, y1: c_float, radius_1: c_float, colors_size: size_t, colors_array: *const size_t, positions_size: size_t, positions_array: *const c_float) -> c_longlong {
    set_gradient_radial(canvas_native_ptr, x0, y0, radius_0, x1, y1, radius_1, colors_size, colors_array, positions_size, positions_array, true)
}


#[no_mangle]
pub extern fn native_set_fill_gradient_linear(canvas_native_ptr: c_longlong, x0: c_float, y0: c_float, x1: c_float, y1: c_float, colors_size: size_t, colors_array: *const size_t, positions_size: size_t, positions_array: *const c_float) -> c_longlong {
    set_gradient_linear(canvas_native_ptr, x0, y0, x1, y1, colors_size, colors_array, positions_size, positions_array, false)
}

#[no_mangle]
pub extern fn native_set_stroke_gradient_linear(canvas_native_ptr: c_longlong, x0: c_float, y0: c_float, x1: c_float, y1: c_float, colors_size: size_t, colors_array: *const size_t, positions_size: size_t, positions_array: *const c_float) -> c_longlong {
    set_gradient_linear(canvas_native_ptr, x0, y0, x1, y1, colors_size, colors_array, positions_size, positions_array, true)
}


#[no_mangle]
pub extern fn native_set_stroke_color_rgba(canvas_native_ptr: c_longlong, red: u8, green: u8, blue: u8, alpha: u8) -> c_longlong {
    set_stroke_color_rgba(canvas_native_ptr, red, green, blue, alpha)
}

#[no_mangle]
pub extern fn native_clear_rect(canvas_native_ptr: c_longlong, x: c_float, y: c_float, width: c_float, height: c_float) -> c_longlong {
    clear_rect(canvas_native_ptr, x, y, width, height)
}

#[no_mangle]
pub extern fn native_clear_canvas(canvas_native_ptr: c_longlong) -> c_longlong {
    clear_canvas(canvas_native_ptr)
}

#[no_mangle]
pub extern fn native_set_line_dash(canvas_native_ptr: c_longlong, size: size_t, array: *const c_float) -> c_longlong {
    set_line_dash(canvas_native_ptr, size, array)
}

#[no_mangle]
pub extern fn native_set_global_composite_operation(canvas_native_ptr: c_longlong, composite: *const c_char) -> c_longlong {
    set_global_composite_operation(canvas_native_ptr, composite)
}

#[no_mangle]
pub extern fn native_set_font(canvas_native_ptr: c_longlong, font: *const c_char) -> c_longlong {
    set_font(canvas_native_ptr, font)
}

#[no_mangle]
pub extern fn native_fill_text(canvas_native_ptr: c_longlong, text: *const c_char, x: c_float, y: c_float, width: c_float) -> c_longlong {
    draw_text(canvas_native_ptr, text, x, y, width, false)
}

#[no_mangle]
pub extern fn native_stroke_text(canvas_native_ptr: c_longlong, text: *const c_char, x: c_float, y: c_float, width: c_float) -> c_longlong {
    draw_text(canvas_native_ptr, text, x, y, width, true)
}

#[no_mangle]
pub extern fn native_scale(canvas_native_ptr: c_longlong, x: c_float, y: c_float) -> c_longlong {
    scale(canvas_native_ptr, x, y)
}

#[no_mangle]
pub extern fn native_transform(canvas_native_ptr: c_longlong, a: c_float, b: c_float, c: c_float, d: c_float, e: c_float, f: c_float) -> c_longlong {
    transform(canvas_native_ptr, a, b, c, d, e, f)
}

#[no_mangle]
pub extern fn native_set_transform(canvas_native_ptr: c_longlong, a: c_float, b: c_float, c: c_float, d: c_float, e: c_float, f: c_float) -> c_longlong {
    set_transform(canvas_native_ptr, a, b, c, d, e, f)
}


#[no_mangle]
pub extern fn native_rotate(canvas_native_ptr: c_longlong, angle: c_float) -> c_longlong {
    rotate(canvas_native_ptr, angle)
}


#[no_mangle]
pub extern fn native_translate(canvas_native_ptr: c_longlong, x: c_float, y: c_float) -> c_longlong {
    translate(canvas_native_ptr, x, y)
}

#[no_mangle]
pub extern fn native_quadratic_curve_to(canvas_native_ptr: c_longlong, cpx: c_float, cpy: c_float, x: c_float, y: c_float) -> c_longlong {
    quadratic_curve_to(canvas_native_ptr, true, cpx, cpy, x, y)
}


#[no_mangle]
pub extern fn native_draw_image(canvas_native_ptr: c_longlong, image_array: *mut u8, image_size: size_t, original_width: c_int, original_height: c_int, dx: c_float, dy: c_float) -> c_longlong {
    draw_image_encoded(canvas_native_ptr, image_array, image_size, original_width, original_height, dx, dy)
}

#[no_mangle]
pub extern fn native_draw_image_dw(canvas_native_ptr: c_longlong, image_array: *mut u8, image_size: size_t, original_width: c_int, original_height: c_int, dx: c_float, dy: c_float, d_width: c_float, d_height: c_float) -> c_longlong {
    draw_image_dw_encoded(canvas_native_ptr, image_array, image_size, original_width, original_height, dx, dy, d_width, d_height)
}


#[no_mangle]
pub extern fn native_draw_image_sw(canvas_native_ptr: c_longlong, image_array: *mut u8, image_size: size_t, original_width: c_int, original_height: c_int, sx: c_float, sy: c_float, s_width: c_float, s_height: c_float, dx: c_float, dy: c_float, d_width: c_float, d_height: c_float) -> c_longlong {
    draw_image_sw_encoded(canvas_native_ptr, image_array, image_size, original_width, original_height, sx, sy, s_width, s_height, dx, dy, d_width, d_height)
}


#[no_mangle]
pub extern fn native_save(canvas_native_ptr: c_longlong) -> c_longlong {
    save(canvas_native_ptr)
}


#[no_mangle]
pub extern fn native_restore(canvas_native_ptr: c_longlong) -> c_longlong {
    restore(canvas_native_ptr)
}

#[no_mangle]
pub extern fn native_measure_text(canvas_native_ptr: c_longlong, text: *const c_char) -> CanvasTextMetrics {
    get_measure_text(canvas_native_ptr, text)
}


#[no_mangle]
pub extern fn native_set_line_cap(canvas_native_ptr: c_longlong, line_cap: *const c_char) -> c_longlong {
    set_line_cap(canvas_native_ptr, line_cap as *mut _)
}

#[no_mangle]
pub extern fn native_set_global_alpha(canvas_native_ptr: c_longlong, alpha: u8) -> c_longlong {
    set_global_alpha(canvas_native_ptr, alpha)
}

#[no_mangle]
pub extern fn native_image_smoothing_enabled(canvas_native_ptr: c_longlong, enabled: bool) -> c_longlong {
    set_image_smoothing_enabled(canvas_native_ptr, enabled)
}

#[no_mangle]
pub extern fn native_image_smoothing_quality(canvas_native_ptr: c_longlong, quality: *const c_char) -> c_longlong {
    set_image_smoothing_quality(canvas_native_ptr, quality)
}

#[no_mangle]
pub extern fn native_line_dash_offset(canvas_native_ptr: c_longlong, offset: f32) -> c_longlong {
    set_line_dash_offset(canvas_native_ptr, offset)
}

#[no_mangle]
pub extern fn native_line_join(canvas_native_ptr: c_longlong, line_cap: *const c_char) -> c_longlong {
    set_line_join(canvas_native_ptr, line_cap)
}

#[no_mangle]
pub extern fn native_miter_limit(canvas_native_ptr: c_longlong, limit: f32) -> c_longlong {
    set_miter_limit(canvas_native_ptr, limit)
}


#[no_mangle]
pub extern fn native_shadow_blur(canvas_native_ptr: c_longlong, limit: f32) -> c_longlong {
    set_shadow_blur(canvas_native_ptr, limit)
}


#[no_mangle]
pub extern fn native_shadow_color(canvas_native_ptr: c_longlong, color: u32) -> c_longlong {
    set_shadow_color(canvas_native_ptr, color)
}


#[no_mangle]
pub extern fn native_shadow_offset_x(canvas_native_ptr: c_longlong, x: f32) -> c_longlong {
    set_shadow_offset_x(canvas_native_ptr, x)
}

#[no_mangle]
pub extern fn native_shadow_offset_y(canvas_native_ptr: c_longlong, y: f32) -> c_longlong {
    set_shadow_offset_y(canvas_native_ptr, y)
}

#[no_mangle]
pub extern fn native_text_align(canvas_native_ptr: c_longlong, alignment: *const c_char) -> c_longlong {
    set_text_align(canvas_native_ptr, alignment)
}

#[no_mangle]
pub extern fn native_reset_transform(canvas_native_ptr: c_longlong) -> c_longlong {
    reset_transform(canvas_native_ptr)
}

#[no_mangle]
pub extern fn native_clip(canvas_native_ptr: c_longlong, fill_rule: *const c_char) -> c_longlong {
    clip_rule(canvas_native_ptr, fill_rule)
}

#[no_mangle]
pub extern fn native_create_image_data(width: size_t, height: size_t) -> CanvasImageData {
    let mut image_data = create_image_data(width as _, height as _);
    CanvasImageData {
        array: image_data.as_ptr() as *const c_void,
        length: image_data.len(),
    }
}

#[no_mangle]
pub extern fn native_put_image_data(canvas_native_ptr: c_longlong, width: size_t, height: size_t, array: *const u8, array_size: size_t, x: c_float, y: c_float, dirty_x: c_float, dirty_y: c_float, dirty_width: size_t, dirty_height: size_t) -> c_longlong {
    let mut slice = unsafe { std::slice::from_raw_parts(array, array_size) };
    put_image_data(canvas_native_ptr, slice.as_ptr(), slice.len(), width as _, height as _, x, y, dirty_x, dirty_y, dirty_width as _, dirty_height as _)
}


#[no_mangle]
pub extern fn native_get_image_data(canvas_native_ptr: c_longlong, sx: c_float, sy: c_float, sw: size_t, sh: size_t) -> CanvasImageData {
    let mut image_data = get_image_data(canvas_native_ptr, sx, sy, sw, sh);
    CanvasImageData {
        array: image_data.1.as_ptr() as *mut c_void,
        length: image_data.1.len(),
    }
}

#[no_mangle]
pub extern fn native_drop_image_data(data: CanvasImageData) {
   let slice = unsafe { std::slice::from_raw_parts(data.array as *const _ as *const u8, data.length) };
    slice.to_vec();
}


#[no_mangle]
pub extern fn native_drop_text_metrics(data: CanvasTextMetrics) {
    Box::new(data);
}


