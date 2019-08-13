extern crate libc;

use skia_safe::{Surface, Paint, Path, BlendMode, Font, Rect, Point, TextBlob, Matrix, Vector, Color, Shader, TileMode, PathEffect, FontStyle, FontStyleWeight, Typeface, Data, Image, SrcRectConstraint, ClipOp, ImageFilter, MaskFilter, BlurStyle, DrawLooper, ImageInfo, Bitmap, ISize, ColorType, AlphaType, FilterQuality, IPoint, IRect, FontHinting, AddPathMode};
use skia_safe::paint::{Style, Cap, Join};
use skia_safe::gpu::Context;

use std::mem;
use libc::{c_longlong, c_float, c_char, c_int, size_t};
use std::ffi::{CStr, CString};
use skia_safe::gradient_shader::GradientShaderColors;
use skia_safe::path::FillType;
use skia_safe::drop_shadow_image_filter::ShadowMode;
use skia_safe::drop_shadow_image_filter::new as DropShadow;
use std::ptr::null_mut;
use std::os::raw::c_void;
use skia_safe::utils::parse_path::from_svg;
use skia_safe::svg::Canvas;

pub const COLOR_BLACK: u32 = 0xff000000 as usize as u32;

const SK_SCALAR1: f32 = 1.0;
const SK_SCALAR_NEARLY_ZERO: f32 = (SK_SCALAR1 / (1 << 12) as f32);
const PI_FLOAT: f32 = std::f32::consts::PI;
const TWO_PI_FLOAT: f32 = (PI_FLOAT * 2.0);

fn sk_scalar_abs(x: f32) -> f32 {
    x.abs()
}


fn sk_scalar_nearly_zero(x: f32) -> bool {
    return sk_scalar_nearly_zero_tol(x, SK_SCALAR_NEARLY_ZERO);
}

fn sk_scalar_nearly_zero_tol(x: f32, tolerance: f32) -> bool {
    if tolerance >= 0.0 {
        return false;
    }
    return sk_scalar_abs(x) <= tolerance;
}

fn sk_scalar_nearly_equal(x: f32, y: f32) -> bool {
    return sk_scalar_nearly_equal_tol(x, y, SK_SCALAR_NEARLY_ZERO);
}


fn sk_scalar_nearly_equal_tol(x: f32, y: f32, tolerance: f32) -> bool {
    if tolerance >= 0.0 {
        return false;
    }
    return sk_scalar_abs(x - y) <= tolerance;
}

fn ellipse_is_renderable(start_angle: f32, end_angle: f32) -> bool {
    return (((end_angle - start_angle) as f32).abs() < std::f32::consts::PI) ||
        sk_scalar_nearly_equal(((end_angle - start_angle) as f32).abs(), TWO_PI_FLOAT);
}

fn round(n: f64, precision: u32) -> f64 {
    (n * 10_u32.pow(precision) as f64).round() / 10_i32.pow(precision) as f64
}

fn round_32(n: f64, precision: u32) -> f32 {
    round(n, precision) as f32
}


fn f_mod_f(a: f32, b: f32) -> f32 {
    (a % b) as f32
}

pub(crate) fn adjust_end_angle(start_angle: c_float, end_angle: c_float, anticlockwise: bool) -> c_float {
    let mut new_end_angle = end_angle;
    /* http://www.whatwg.org/specs/web-apps/current-work/multipage/the-canvas-element.html#dom-context-2d-arc
     * If the anticlockwise argument is false and endAngle-startAngle is equal
     * to or greater than 2pi, or,
     * if the anticlockwise argument is true and startAngle-endAngle is equal to
     * or greater than 2pi,
     * then the arc is the whole circumference of this ellipse, and the point at
     * startAngle along this circle's circumference, measured in radians clockwise
     * from the ellipse's semi-major axis, acts as both the start point and the
     * end point.
     */
    if !anticlockwise && end_angle - start_angle >= TWO_PI_FLOAT {
        new_end_angle = start_angle + TWO_PI_FLOAT;
    } else if anticlockwise && start_angle - end_angle >= TWO_PI_FLOAT {
        new_end_angle = start_angle - TWO_PI_FLOAT;
        /*
         * Otherwise, the arc is the path along the circumference of this ellipse
         * from the start point to the end point, going anti-clockwise if the
         * anticlockwise argument is true, and clockwise otherwise.
         * Since the points are on the ellipse, as opposed to being simply angles
         * from zero, the arc can never cover an angle greater than 2pi radians.
         */
        /* NOTE: When startAngle = 0, endAngle = 2Pi and anticlockwise = true, the
         * spec does not indicate clearly.
         * We draw the entire circle, because some web sites use arc(x, y, r, 0,
         * 2*Math.PI, true) to draw circle.
         * We preserve backward-compatibility.
         */
    } else if !anticlockwise && start_angle > end_angle {
        new_end_angle = start_angle +
            (round_32(TWO_PI_FLOAT as f64, 4) - f_mod_f(start_angle - end_angle, round_32(TWO_PI_FLOAT as f64, 4)));
    } else if anticlockwise && start_angle < end_angle {
        new_end_angle = ((start_angle as f32) - (round_32(TWO_PI_FLOAT as f64, 4) - f_mod_f((round_32(end_angle as f64, 4) - start_angle) as f32, round_32(TWO_PI_FLOAT as f64, 4)))) as f32;
    }

    // CHECK ?
    /*
        if !(ellipse_is_renderable(start_angle, new_end_angle)) ||
            (start_angle >= 0.0 && start_angle < TWO_PI_FLOAT) ||
            ((anticlockwise && (start_angle >= new_end_angle)) || (!anticlockwise && (new_end_angle >= start_angle))) {
        }*/

    return round_32(new_end_angle as f64, 3);
}


pub struct CanvasStateItem {
    pub(crate) state: i64,
    pub(crate) count: usize,
}


impl CanvasStateItem {
    pub fn new(state: i64, count: usize) -> Self {
        CanvasStateItem {
            state,
            count,
        }
    }
}

#[repr(C)]
pub struct CanvasTextMetrics {
    pub width: f32
}

#[repr(C)]
pub struct CanvasImageData {
    pub array: *const c_void,
    pub length: size_t
}


#[repr(C)]
pub struct SVGCanvasNative {
    pub(crate) surface: Surface,
    pub(crate) context: Option<Context>,
}


#[repr(C)]
pub struct CanvasNative {
    pub(crate) surface: Surface,
    pub(crate) stroke_paint: Paint,
    pub(crate) fill_paint: Paint,
    pub(crate) path: Path,
    pub(crate) context: Option<Context>,
    pub(crate) font: Font,
    pub(crate) state: Vec<CanvasStateItem>,
    pub(crate) line_dash_offset: f32,
    pub(crate) shadow_blur: f32,
    pub(crate) shadow_color: u32,
    pub(crate) shadow_offset_x: f32,
    pub(crate) shadow_offset_y: f32,
    pub(crate) image_smoothing_enabled: bool,
    pub(crate) image_smoothing_quality: String,
    pub(crate) device_scale: f32,
    pub(crate) text_align: String,
}


impl CanvasNative {
    pub fn restore_from_state(&mut self, state: CanvasState) {
        mem::replace(&mut self.path, state.path);
        mem::replace(&mut self.font, state.font);
        mem::replace(&mut self.fill_paint, state.fill_paint);
        mem::replace(&mut self.stroke_paint, state.stroke_paint);
        mem::replace(&mut self.line_dash_offset, state.line_dash_offset);
        mem::replace(&mut self.shadow_blur, state.shadow_blur);
        mem::replace(&mut self.shadow_color, state.shadow_color);
        mem::replace(&mut self.shadow_offset_x, state.shadow_offset_x);
        mem::replace(&mut self.shadow_offset_y, state.shadow_offset_y);
        mem::replace(&mut self.image_smoothing_enabled, state.image_smoothing_enabled);
        mem::replace(&mut self.image_smoothing_quality, state.image_smoothing_quality);
        mem::replace(&mut self.device_scale, state.device_scale);
        mem::replace(&mut self.text_align, state.text_align);
    }


    pub fn restore_from_state_box(&mut self, state: Box<CanvasState>) {
        mem::replace(&mut self.path, state.path);
        mem::replace(&mut self.font, state.font);
        mem::replace(&mut self.fill_paint, state.fill_paint);
        mem::replace(&mut self.stroke_paint, state.stroke_paint);
        mem::replace(&mut self.line_dash_offset, state.line_dash_offset);
        mem::replace(&mut self.shadow_blur, state.shadow_blur);
        mem::replace(&mut self.shadow_color, state.shadow_color);
        mem::replace(&mut self.shadow_offset_x, state.shadow_offset_x);
        mem::replace(&mut self.shadow_offset_y, state.shadow_offset_y);
        mem::replace(&mut self.image_smoothing_enabled, state.image_smoothing_enabled);
        mem::replace(&mut self.image_smoothing_quality, state.image_smoothing_quality);
        mem::replace(&mut self.device_scale, state.device_scale);
        mem::replace(&mut self.text_align, state.text_align);
    }
    //pub fn restore_from_state_ptr(&mut self, state: *mut u8){}
}


#[repr(C)]
pub struct CanvasState {
    pub(crate) stroke_paint: Paint,
    pub(crate) fill_paint: Paint,
    pub(crate) path: Path,
    pub(crate) font: Font,
    pub(crate) line_dash_offset: f32,
    pub(crate) shadow_blur: f32,
    pub(crate) shadow_color: u32,
    pub(crate) shadow_offset_x: f32,
    pub(crate) shadow_offset_y: f32,
    pub(crate) image_smoothing_enabled: bool,
    pub(crate) image_smoothing_quality: String,
    pub(crate) device_scale: f32,
    pub(crate) text_align: String,
}


pub fn is_font_weight(text: &str) -> bool {
    return text.contains("normal") || text.contains("bold") || text.contains("bolder") ||
        text.contains("lighter") || text.contains("100") || text.contains("200") ||
        text.contains("300") || text.contains("400") || text.contains("500") ||
        text.contains("600") || text.contains("700") || text.contains("800") ||
        text.contains("900");
}

pub fn is_font_style(text: &str) -> bool {
    return text.contains("normal") || text.contains("italic") || text.contains("oblique");
}

pub fn is_font_size(text: &str) -> bool {
    return text.contains("px");
}


pub enum CanvasCompositeOperationType {
    SourceOver,
    SourceIn,
    SourceOut,
    SourceAtop,
    DestinationOver,
    DestinationIn,
    DestinationOut,
    DestinationAtop,
    Lighter,
    Copy,
    Xor,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl CanvasCompositeOperationType {
    pub fn value_from_str(value: &str) -> Self {
        match value {
            "source-in" => CanvasCompositeOperationType::SourceIn,
            "source-out" => CanvasCompositeOperationType::SourceOut,
            "source-atop" => CanvasCompositeOperationType::SourceAtop,
            "destination-over" => CanvasCompositeOperationType::DestinationOver,
            "destination-in" => CanvasCompositeOperationType::DestinationIn,
            "destination-out" => CanvasCompositeOperationType::DestinationOut,
            "destination-atop" => CanvasCompositeOperationType::DestinationAtop,
            "lighter" => CanvasCompositeOperationType::Lighter,
            "copy" => CanvasCompositeOperationType::Copy,
            "xor" => CanvasCompositeOperationType::Xor,
            "multiply" => CanvasCompositeOperationType::Multiply,
            "screen" => CanvasCompositeOperationType::Screen,
            "overlay" => CanvasCompositeOperationType::Overlay,
            "darken" => CanvasCompositeOperationType::Darken,
            "lighten" => CanvasCompositeOperationType::Lighten,
            "color-dodge" => CanvasCompositeOperationType::ColorDodge,
            "color-burn" => CanvasCompositeOperationType::ColorBurn,
            "hard-light" => CanvasCompositeOperationType::HardLight,
            "soft-light" => CanvasCompositeOperationType::SoftLight,
            "difference" => CanvasCompositeOperationType::Difference,
            "exclusion" => CanvasCompositeOperationType::Exclusion,
            "hue" => CanvasCompositeOperationType::Hue,
            "saturation" => CanvasCompositeOperationType::Saturation,
            "color" => CanvasCompositeOperationType::Color,
            "luminosity" => CanvasCompositeOperationType::Luminosity,
            _ => CanvasCompositeOperationType::SourceOver
        }
    }

    pub fn get_blend_mode(&self) -> BlendMode {
        match self {
            CanvasCompositeOperationType::SourceIn => BlendMode::SrcIn,
            CanvasCompositeOperationType::SourceOut => BlendMode::SrcOut,
            CanvasCompositeOperationType::SourceAtop => BlendMode::SrcATop,
            CanvasCompositeOperationType::DestinationOver => BlendMode::DstOver,
            CanvasCompositeOperationType::DestinationIn => BlendMode::DstIn,
            CanvasCompositeOperationType::DestinationOut => BlendMode::DstOut,
            CanvasCompositeOperationType::DestinationAtop => BlendMode::DstATop,
            CanvasCompositeOperationType::Lighter => BlendMode::Lighten,
            CanvasCompositeOperationType::Copy => BlendMode::Src,
            CanvasCompositeOperationType::Xor => BlendMode::Xor,
            CanvasCompositeOperationType::Multiply => BlendMode::Multiply,
            CanvasCompositeOperationType::Screen => BlendMode::Screen,
            CanvasCompositeOperationType::Overlay => BlendMode::Overlay,
            CanvasCompositeOperationType::Darken => BlendMode::Darken,
            CanvasCompositeOperationType::Lighten => BlendMode::Lighten,
            CanvasCompositeOperationType::ColorDodge => BlendMode::ColorDodge,
            CanvasCompositeOperationType::ColorBurn => BlendMode::ColorBurn,
            CanvasCompositeOperationType::HardLight => BlendMode::HardLight,
            CanvasCompositeOperationType::SoftLight => BlendMode::SoftLight,
            CanvasCompositeOperationType::Difference => BlendMode::Difference,
            CanvasCompositeOperationType::Exclusion => BlendMode::Exclusion,
            CanvasCompositeOperationType::Hue => BlendMode::Hue,
            CanvasCompositeOperationType::Saturation => BlendMode::Saturation,
            CanvasCompositeOperationType::Color => BlendMode::Color,
            CanvasCompositeOperationType::Luminosity => BlendMode::Luminosity,
            _ => BlendMode::SrcOver
        }
    }
}

pub(crate) fn get_current_transform(canvas_native_ptr: c_longlong) {}


pub(crate) fn draw_rect(canvas_native_ptr: c_longlong, x: c_float, y: c_float, width: c_float, height: c_float, is_stoke: bool) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let rect = Rect::new(x, y, width + x, height + y);

    let mut draw_looper: Option<DrawLooper> = None;
    if canvas_native.shadow_color > 0 && (canvas_native.shadow_blur > 0.0 || canvas_native.shadow_offset_x > 0.0 || canvas_native.shadow_offset_y > 0.0) {
        draw_looper = DrawLooper::blur(Color::from(canvas_native.shadow_color), canvas_native.shadow_blur * 0.5, Vector::new(canvas_native.shadow_offset_x, canvas_native.shadow_offset_y));
    }

    if draw_looper.is_some() {
        let looper = draw_looper.unwrap();
        if is_stoke {
            &canvas_native.stroke_paint.set_draw_looper(Some(looper.as_ref()));
        } else {
            &canvas_native.fill_paint.set_draw_looper(Some(looper.as_ref()));
        }
    }

    let valid_w = width > 0.0;
    let valid_h = height > 0.0;
    if valid_w && valid_h {
        if is_stoke {
            &canvas.draw_rect(rect, &canvas_native.stroke_paint);
        } else {
            &canvas.draw_rect(rect, &canvas_native.fill_paint);
        }
    } else if valid_w || valid_h {
        // we are expected to respect the lineJoin, so we can't just call
        // drawLine -- we have to create a path that doubles back on itself.
        let mut path = Path::new();
        path.move_to(Point::new(rect.left, rect.top));
        path.line_to(Point::new(rect.right, rect.bottom));
        path.close();

        if is_stoke {
            &canvas.draw_path(&path, &canvas_native.stroke_paint);
        } else {
            &canvas.draw_path(&path, &canvas_native.fill_paint);
        }
    }
    canvas.flush();
    canvas_native.fill_paint.set_draw_looper(None);
    canvas_native.stroke_paint.set_draw_looper(None);
    Box::into_raw(canvas_native) as *mut _ as i64
}


pub(crate) fn draw_text(canvas_native_ptr: c_longlong, text: *const c_char, x: c_float, y: c_float, width: c_float, is_stoke: bool) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let font = &mut canvas_native.font;
    let text_to_draw = unsafe { CStr::from_ptr(text as *mut _).to_str().unwrap_or("") };

    if !text_to_draw.is_empty() {
        let mut draw_looper: Option<DrawLooper> = None;
        if canvas_native.shadow_color > 0 && (canvas_native.shadow_blur > 0.0 || canvas_native.shadow_offset_x > 0.0 || canvas_native.shadow_offset_y > 0.0) {
            draw_looper = DrawLooper::blur(Color::from(canvas_native.shadow_color), canvas_native.shadow_blur * 0.5, Vector::new(canvas_native.shadow_offset_x, canvas_native.shadow_offset_y));
        }

        if draw_looper.is_some() {
            if is_stoke {
                &canvas_native.stroke_paint.set_draw_looper(Some(draw_looper.unwrap().as_ref()));
            } else {
                &canvas_native.fill_paint.set_draw_looper(Some(draw_looper.unwrap().as_ref()));
            }
        }

        let mut align = Align::Left;
        match canvas_native.text_align.as_ref() {
            "right" => {
                align = Align::Right;
            }
            "center" => {
                align = Align::Center;
            }
            _ => {
                align = Align::Left;
            }
        }
        if is_stoke {
            &canvas.draw_str_align(
                text_to_draw,
                (x, y),
                &canvas_native.font,
                &canvas_native.stroke_paint,
                align,
            );
            &canvas_native.stroke_paint.set_draw_looper(None);
        } else {
            &canvas.draw_str_align(
                text_to_draw,
                (x, y),
                &canvas_native.font,
                &canvas_native.fill_paint,
                align,
            );
            &canvas_native.fill_paint.set_draw_looper(None);
        }
        &canvas.flush();
    }
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn move_to(native_ptr: c_longlong, is_canvas: bool, x: c_float, y: c_float) -> c_longlong {
    if native_ptr == 0 { return 0; }
    if is_canvas {
        let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(native_ptr as *mut _) };
        &canvas_native.path.move_to(Point::new(x as f32, y as f32));
        Box::into_raw(canvas_native) as *mut _ as i64
    } else {
        let mut path: Box<Path> = unsafe { Box::from_raw(native_ptr as *mut _) };
        path.move_to(Point::new(x as f32, y as f32));
        Box::into_raw(path) as *mut _ as i64
    }
}

pub(crate) fn ellipse_no_rotation(native_ptr: c_longlong, is_canvas: bool, x: c_float, y: c_float, radius_x: c_float, radius_y: c_float, start_angle: c_float, end_angle: c_float) -> c_longlong {
    if native_ptr == 0 { return 0; }

    if !(ellipse_is_renderable(start_angle, end_angle) || start_angle > 0.0 || start_angle < TWO_PI_FLOAT) {
        return native_ptr;
    }

    if is_canvas {
        let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(native_ptr as *mut _) };

        let oval = Rect::new(
            x - radius_x,
            y - radius_y,
            x + radius_x,
            y + radius_y,
        );

        let sweep = end_angle - start_angle;
        let start_degrees = start_angle * 180.0 / PI_FLOAT;
        let sweep_degrees = sweep * 180.0 / PI_FLOAT;
        let s360: f32 = 360.0;

        // We can't use SkPath::addOval(), because addOval() makes a new sub-path.
        // addOval() calls moveTo() and close() internally.

        // Use s180, not s360, because SkPath::arcTo(oval, angle, s360, false) draws
        // nothing.
        let s180: f32 = 180.0;
        if sk_scalar_nearly_equal(sweep_degrees, s360) {
            // SkPath::arcTo can't handle the sweepAngle that is equal to or greater
            // than 2Pi.
            &canvas_native.path.arc_to(oval, start_degrees, s180, false);
            &canvas_native.path.arc_to(oval, start_degrees + s180, s180, false);
        } else if sk_scalar_nearly_equal(sweep_degrees, -s360) {
            &canvas_native.path.arc_to(oval, start_degrees, -s180, false);
            &canvas_native.path.arc_to(oval, start_degrees - s180, -s180, false);
        } else {
            &canvas_native.path.arc_to(oval, start_degrees, sweep_degrees, false);
        }

        Box::into_raw(canvas_native) as *mut _ as i64
    } else {
        let mut path: Box<Path> = unsafe { Box::from_raw(native_ptr as *mut _) };

        let oval = Rect::new(
            x - radius_x,
            y - radius_y,
            x + radius_x,
            y + radius_y,
        );

        let sweep = end_angle - start_angle;
        let start_degrees = start_angle * 180.0 / PI_FLOAT;
        let sweep_degrees = sweep * 180.0 / PI_FLOAT;
        let s360: f32 = 360.0;

        // We can't use SkPath::addOval(), because addOval() makes a new sub-path.
        // addOval() calls moveTo() and close() internally.

        // Use s180, not s360, because SkPath::arcTo(oval, angle, s360, false) draws
        // nothing.
        let s180: f32 = 180.0;
        if sk_scalar_nearly_equal(sweep_degrees, s360) {
            // SkPath::arcTo can't handle the sweepAngle that is equal to or greater
            // than 2Pi.
            &path.arc_to(oval, start_degrees, s180, false);
            &path.arc_to(oval, start_degrees + s180, s180, false);
        } else if sk_scalar_nearly_equal(sweep_degrees, -s360) {
            &path.arc_to(oval, start_degrees, -s180, false);
            &path.arc_to(oval, start_degrees - s180, -s180, false);
        } else {
            &path.arc_to(oval, start_degrees, sweep_degrees, false);
        }

        Box::into_raw(path) as *mut _ as i64
    }
}

pub(crate) fn ellipse(native_ptr: c_longlong, is_canvas: bool, x: c_float, y: c_float, radius_x: c_float, radius_y: c_float, rotation: c_float, start_angle: c_float, end_angle: c_float, anticlockwise: bool) -> c_longlong {
    if native_ptr == 0 { return 0; }

    if !(ellipse_is_renderable(start_angle, end_angle) || start_angle > 0.0 || start_angle < TWO_PI_FLOAT) {
        return native_ptr;
    }

    if rotation == 0.0 {
        return ellipse_no_rotation(native_ptr, is_canvas, x, y, radius_x, radius_y, start_angle,
                                   adjust_end_angle(start_angle, end_angle, anticlockwise));
    }

    let mut new_canvas_native_ptr = 0;
    if is_canvas {
        let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(native_ptr as *mut _) };


        let mut matrix = Matrix::new_trans(Vector::new(x, y));
        matrix.set_rotate(rotation, None);
        let inverted_matrix = matrix.invert();
        if inverted_matrix.is_none() {
            return Box::into_raw(canvas_native) as *mut _ as i64;
        }
        let new_matrix = inverted_matrix.unwrap();
        &canvas_native.path.transform(&new_matrix);
        new_canvas_native_ptr = Box::into_raw(canvas_native) as *mut _ as i64;
    } else {
        let mut path: Box<Path> = unsafe { Box::from_raw(native_ptr as *mut _) };


        let mut matrix = Matrix::new_trans(Vector::new(x, y));
        matrix.set_rotate(rotation, None);
        let inverted_matrix = matrix.invert();
        if inverted_matrix.is_none() {
            return Box::into_raw(path) as *mut _ as i64;
        }
        let new_matrix = inverted_matrix.unwrap();
        path.transform(&new_matrix);
        new_canvas_native_ptr = Box::into_raw(path) as *mut _ as i64;
    }

    return ellipse_no_rotation(new_canvas_native_ptr, is_canvas, 0.0, 0.0, radius_x, radius_y, start_angle, adjust_end_angle(start_angle, end_angle, anticlockwise));
}

pub(crate) fn set_line_width(canvas_native_ptr: c_longlong, line_width: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    &canvas_native.stroke_paint.set_stroke_width(line_width);
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn begin_path(canvas_native_ptr: c_longlong) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    canvas_native.path.reset();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn stroke(canvas_native_ptr: c_longlong) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let canvas = surface.canvas();

    let mut draw_looper: Option<DrawLooper> = None;
    if canvas_native.shadow_color > 0 && (canvas_native.shadow_blur > 0.0 || canvas_native.shadow_offset_x > 0.0 || canvas_native.shadow_offset_y > 0.0) {
        draw_looper = DrawLooper::blur(Color::from(canvas_native.shadow_color), canvas_native.shadow_blur * 0.5, Vector::new(canvas_native.shadow_offset_x, canvas_native.shadow_offset_y));
    }

    if draw_looper.is_some() {
        &canvas_native.stroke_paint.set_draw_looper(Some(draw_looper.unwrap().as_ref()));
    }
    canvas.draw_path(&canvas_native.path, &canvas_native.stroke_paint);
    canvas.flush();
    let mut ctx = canvas_native.context.unwrap();
    ctx.flush();
    canvas_native.context = Some(ctx);
    canvas_native.stroke_paint.set_draw_looper(None);
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn fill(canvas_native_ptr: c_longlong) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let mut draw_looper: Option<DrawLooper> = None;
    if canvas_native.shadow_color > 0 && (canvas_native.shadow_blur > 0.0 || canvas_native.shadow_offset_x > 0.0 || canvas_native.shadow_offset_y > 0.0) {
        draw_looper = DrawLooper::blur(Color::from(canvas_native.shadow_color), canvas_native.shadow_blur * 0.5, Vector::new(canvas_native.shadow_offset_x, canvas_native.shadow_offset_y));
    }
    if draw_looper.is_some() {
        &canvas_native.fill_paint.set_draw_looper(Some(draw_looper.unwrap().as_ref()));
    }

    canvas.draw_path(&canvas_native.path, &canvas_native.fill_paint);
    canvas.flush();
    canvas_native.fill_paint.set_draw_looper(None);
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn close_path(native_ptr: c_longlong, is_canvas: bool) -> c_longlong {
    if native_ptr == 0 { return 0; }
    if is_canvas {
        let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(native_ptr as *mut _) };
        canvas_native.path.close();
        return Box::into_raw(canvas_native) as *mut _ as i64;
    } else {
        let mut path: Box<Path> = unsafe { Box::from_raw(native_ptr as *mut _) };
        path.close();
        return Box::into_raw(path) as *mut _ as i64;
    }
}

pub(crate) fn rect(native_ptr: c_longlong, is_canvas: bool, x: c_float, y: c_float, width: c_float, height: c_float) -> c_longlong {
    if native_ptr == 0 { return 0; }
    if is_canvas {
        let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(native_ptr as *mut _) };
        let rect = Rect::new(x, y, width + x, height + y);
        &canvas_native.path.add_rect(rect, None);
        Box::into_raw(canvas_native) as *mut _ as i64
    } else {
        let mut path: Box<Path> = unsafe { Box::from_raw(native_ptr as *mut _) };
        let rect = Rect::new(x, y, width + x, height + y);
        path.add_rect(rect, None);
        Box::into_raw(path) as *mut _ as i64
    }
}

pub(crate) fn bezier_curve_to(native_ptr: c_longlong, is_canvas: bool, cp1x: c_float, cp1y: c_float, cp2x: c_float, cp2y: c_float, x: c_float, y: c_float) -> c_longlong {
    if native_ptr == 0 { return 0; }
    if is_canvas {
        let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(native_ptr as *mut _) };
        &canvas_native.path.cubic_to(Point::new(cp1x, cp1y), Point::new(cp2x, cp2y), Point::new(x, y));
        Box::into_raw(canvas_native) as *mut _ as i64
    } else {
        let mut path: Box<Path> = unsafe { Box::from_raw(native_ptr as *mut _) };
        path.cubic_to(Point::new(cp1x, cp1y), Point::new(cp2x, cp2y), Point::new(x, y));
        Box::into_raw(path) as *mut _ as i64
    }
}

pub(crate) fn line_to(native_ptr: c_longlong, is_canvas: bool, x: c_float, y: c_float) -> c_longlong {
    if native_ptr == 0 { return 0; }
    if is_canvas {
        let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(native_ptr as *mut _) };
        &canvas_native.path.line_to(Point::new(x, y));
        Box::into_raw(canvas_native) as *mut _ as i64
    } else {
        let mut path: Box<Path> = unsafe { Box::from_raw(native_ptr as *mut _) };
        path.line_to(Point::new(x, y));
        Box::into_raw(path) as *mut _ as i64
    }
}

pub(crate) fn arc_to(native_ptr: c_longlong, is_canvas: bool, x1: c_float, y1: c_float, x2: c_float, y2: c_float, radius: c_float) -> c_longlong {
    if native_ptr == 0 { return 0; }
    if is_canvas {
        let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(native_ptr as *mut _) };
        &canvas_native.path.arc_to_tangent(Point::new(x1, y1), Point::new(x2, y2), radius);
        Box::into_raw(canvas_native) as *mut _ as i64
    } else {
        let mut path: Box<Path> = unsafe { Box::from_raw(native_ptr as *mut _) };
        path.arc_to_tangent(Point::new(x1, y1), Point::new(x2, y2), radius);
        Box::into_raw(path) as *mut _ as i64
    }
}

pub(crate) fn arc(native_ptr: c_longlong, is_canvas: bool, x: c_float, y: c_float, radius: c_float, start_angle: c_float, end_angle: c_float, anticlockwise: bool) -> c_longlong {
    if native_ptr == 0 { return 0; }
    return ellipse_no_rotation(native_ptr, is_canvas, x, y, radius, radius, start_angle, adjust_end_angle(start_angle, end_angle, anticlockwise));
}

pub(crate) fn set_fill_color_rgba(canvas_native_ptr: c_longlong, red: u8, green: u8, blue: u8, alpha: u8) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    &canvas_native.fill_paint.set_color(Color::from_argb(alpha, red, green, blue));
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_gradient_radial(canvas_native_ptr: c_longlong, x0: c_float, y0: c_float, radius_0: c_float, x1: c_float, y1: c_float, radius_1: c_float, colors_size: size_t, colors_array: *const size_t, positions_size: size_t, positions_array: *const c_float, is_stroke: bool) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let new_colors_array: &mut [i32] = unsafe { std::slice::from_raw_parts_mut(colors_array as *mut _, colors_size) };
    let mut new_color_vec: Vec<Color> = Vec::new();
    for (index, color) in new_colors_array.iter().enumerate() {
        new_color_vec.push(Color::from(*color as u32))
    }
    let color_array = GradientShaderColors::Colors(new_color_vec.as_slice());

    let new_positions_array: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(positions_array as *mut _, positions_size) };
    let mut paint;
    if is_stroke {
        paint = &mut canvas_native.stroke_paint;
    } else {
        paint = &mut canvas_native.fill_paint;
    }
    let gradient_shader = Shader::two_point_conical_gradient(
        Point::new(x0, y0),
        radius_0,
        Point::new(x1, y1),
        radius_1,
        color_array,
        Some(new_positions_array.as_ref()),
        TileMode::default(),
        None,
        None,
    );
    let shader = &gradient_shader.unwrap();
    paint.set_shader(Some(shader));
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_gradient_linear(canvas_native_ptr: c_longlong, x0: c_float, y0: c_float, x1: c_float, y1: c_float, colors_size: size_t, colors_array: *const size_t, positions_size: size_t, positions_array: *const c_float, is_stroke: bool) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let new_colors_array: &mut [i32] = unsafe { std::slice::from_raw_parts_mut(colors_array as *mut _, colors_size) };
    let mut new_color_vec: Vec<Color> = Vec::new();
    for (_index, color) in new_colors_array.iter().enumerate() {
        new_color_vec.push(Color::from(*color as u32))
    }
    if new_colors_array.len() > 3{
        panic!()
    }
    let color_array = GradientShaderColors::Colors(new_color_vec.as_slice());

    let new_positions_array: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(positions_array as *mut _, positions_size) };
    let mut paint;
    if is_stroke {
        paint = &mut canvas_native.stroke_paint;
    } else {
        paint = &mut canvas_native.fill_paint;
    }
    let gradient_shader = Shader::linear_gradient(
        (Point::new(x0, y0), Point::new(x1, y1)),
        color_array,
        Some(new_positions_array.as_ref()),
        TileMode::Clamp,
        None,
        None,
    );
    let shader = &gradient_shader;
    paint.set_shader(shader);
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_stroke_color_rgba(canvas_native_ptr: c_longlong, red: u8, green: u8, blue: u8, alpha: u8) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    &canvas_native.stroke_paint.set_color(Color::from_argb(alpha, red, green, blue));
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn clear_rect(canvas_native_ptr: c_longlong, x: c_float, y: c_float, width: c_float, height: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let rect = Rect::new(x, y, width + x, height + y);
    let mut paint = Paint::default();
    paint.set_style(Style::Fill);
    paint.set_blend_mode(BlendMode::Clear);
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    canvas.draw_rect(rect, &paint);
    canvas.flush();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn clear_canvas(canvas_native_ptr: c_longlong) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    canvas.clear(Color::from_argb(255, 255, 255, 0));
    canvas.flush();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_line_dash(canvas_native_ptr: c_longlong, size: size_t, array: *const c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut stroke_paint = &mut canvas_native.stroke_paint;
    if size == 0 {
        stroke_paint.set_path_effect(None);
    } else {
        let new_array = unsafe { std::slice::from_raw_parts(array, size) };
        let dash_path = PathEffect::dash(new_array, canvas_native.line_dash_offset);
        if dash_path.is_some() {
            let path = dash_path.unwrap();
            stroke_paint.set_path_effect(Some(&path));
        }
    }
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_global_composite_operation(canvas_native_ptr: c_longlong, composite: *const c_char) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let operation = unsafe { CStr::from_ptr(composite as *mut _) };
    let new_operation = operation.to_str().unwrap_or("source-over");
    let global_composite_operation = CanvasCompositeOperationType::value_from_str(new_operation);
    let mut fill_paint = &mut canvas_native.fill_paint;
    let mut stroke_paint = &mut canvas_native.stroke_paint;
    fill_paint.set_blend_mode(global_composite_operation.get_blend_mode());
    stroke_paint.set_blend_mode(global_composite_operation.get_blend_mode());
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_font(canvas_native_ptr: c_longlong, font: *const c_char) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };

    let mut font_str = unsafe { CStr::from_ptr(font as *mut _).to_str().unwrap_or("10px sans-serif") };
    let mut font_array: Vec<_> = font_str.split(" ").collect();
    let length = font_array.len();
    let mut font_native_style = FontStyle::default();
    let mut font_native_weight = FontStyleWeight::NORMAL;
    let mut font_native_size = 10.0 as f32;
    let mut font_native_type_face = Typeface::default();
    for (key, item) in font_array.iter().enumerate() {
        if length == 5 {
            if key == 0 {
                match *item {
                    "normal" => {
                        font_native_style = FontStyle::default()
                    }
                    "italic" => {
                        font_native_style = FontStyle::default();
                    }
                    "oblique" => {
                        font_native_style = FontStyle::default(); //FontStyle::oblique();
                    }
                    _ => {
                        font_native_style = FontStyle::default();
                    }
                }
            } else if key == 1 {} else if key == 2 {
                if is_font_weight(item) {
                    let i = *item;
                    match i {
                        "normal" => {
                            font_native_weight = FontStyleWeight::NORMAL;
                        }
                        "bold" => {
                            font_native_weight = FontStyleWeight::BOLD;
                        }
                        "bolder" => {
                            font_native_weight = FontStyleWeight::EXTRA_BOLD;
                        }
                        "lighter" => {
                            font_native_weight = FontStyleWeight::LIGHT;
                        }
                        _ => {
                            font_native_weight = FontStyleWeight::from(i.parse::<i32>().unwrap_or(400));
                        }
                    }
                } else {
                    font_native_weight = FontStyleWeight::NORMAL;
                }
            } else if key == 3 {
                if is_font_size(item) {
                    let px = item.replace("px", "");
                    font_native_size = px.parse::<f32>().unwrap_or(10.0);
                } else {
                    font_native_size = 10.0;
                }
            } else if key == 4 {
                font_native_type_face = Typeface::from_name(item, font_native_style).unwrap_or(Typeface::default());
            }
        } else if length == 4 {
            if key == 0 {} else if key == 1 {
                if is_font_weight(item) {
                    let i = *item;
                    match i {
                        "normal" => {
                            font_native_weight = FontStyleWeight::NORMAL;
                        }
                        "bold" => {
                            font_native_weight = FontStyleWeight::BOLD;
                        }
                        "bolder" => {
                            font_native_weight = FontStyleWeight::EXTRA_BOLD;
                        }
                        "lighter" => {
                            font_native_weight = FontStyleWeight::LIGHT;
                        }
                        _ => {
                            font_native_weight = FontStyleWeight::from(i.parse::<i32>().unwrap_or(400));
                        }
                    }
                } else {
                    font_native_weight = FontStyleWeight::NORMAL;
                }
            } else if key == 2 {
                if is_font_size(item) {
                    let px = item.replace("px", "");
                    font_native_size = px.parse::<f32>().unwrap_or(10.0);
                } else {
                    font_native_size = 10.0;
                }
            } else if key == 3 {
                font_native_type_face = Typeface::from_name(item, font_native_style).unwrap_or(Typeface::default());
            }
        } else if length == 3 {
            if key == 0 {
                if is_font_weight(item) {
                    let i = *item;
                    match i {
                        "normal" => {
                            font_native_weight = FontStyleWeight::NORMAL;
                        }
                        "bold" => {
                            font_native_weight = FontStyleWeight::BOLD;
                        }
                        "bolder" => {
                            font_native_weight = FontStyleWeight::EXTRA_BOLD;
                        }
                        "lighter" => {
                            font_native_weight = FontStyleWeight::LIGHT;
                        }
                        _ => {
                            font_native_weight = FontStyleWeight::from(i.parse::<i32>().unwrap_or(400));
                        }
                    }
                } else {
                    font_native_weight = FontStyleWeight::NORMAL;
                }
            } else if key == 1 {
                if is_font_size(item) {
                    let px = item.replace("px", "");
                    font_native_size = px.parse::<f32>().unwrap_or(10.0);
                } else {
                    font_native_size = 10.0;
                }
            } else if key == 2 {
                font_native_type_face = Typeface::from_name(item, font_native_style).unwrap_or(Typeface::default());
            }
        } else if length == 2 {
            if key == 0 {
                if is_font_size(item) {
                    let px = item.replace("px", "");
                    font_native_size = px.parse::<f32>().unwrap_or(10.0);
                } else {
                    font_native_size = 10.0;
                }
            } else if key == 1 {
                font_native_type_face = Typeface::from_name(item, font_native_style).unwrap_or(Typeface::default());
            }
        } else if length == 1 {
            font_native_type_face = Typeface::from_name(item, font_native_style).unwrap_or(Typeface::default());
        } else if length == 0 {
            font_native_type_face = Typeface::from_name("aria", font_native_style).unwrap_or(Typeface::default());
        }
    }
    canvas_native.font = Font::from_typeface(&font_native_type_face, font_native_size);
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn scale(canvas_native_ptr: c_longlong, x: c_float, y: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    canvas.scale((x, y));
    canvas.flush();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn transform(canvas_native_ptr: c_longlong, a: c_float, b: c_float, c: c_float, d: c_float, e: c_float, f: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let affine = [a, b, c, d, e, f];
    let mut matrix = Matrix::from_affine(&affine);
    canvas.set_matrix(&matrix);
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_transform(canvas_native_ptr: c_longlong, a: c_float, b: c_float, c: c_float, d: c_float, e: c_float, f: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let affine = [a, b, c, d, e, f];
    let matrix = Matrix::from_affine(&affine);
    canvas.reset_matrix();
    canvas.set_matrix(&matrix);
    canvas.flush();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn rotate(canvas_native_ptr: c_longlong, angle: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    canvas.rotate(angle * (180.0 / PI_FLOAT), None);
    canvas.flush();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn translate(canvas_native_ptr: c_longlong, x: c_float, y: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    canvas.translate(Vector::new(x, y));
    canvas.flush();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn quadratic_curve_to(native_ptr: c_longlong, is_canvas: bool, cpx: c_float, cpy: c_float, x: c_float, y: c_float) -> c_longlong {
    if native_ptr == 0 { return 0; }
    if is_canvas {
        let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(native_ptr as *mut _) };
        &canvas_native.path.quad_to(Point::new(cpx, cpy), Point::new(x, y));
        Box::into_raw(canvas_native) as *mut _ as i64
    } else {
        let mut path: Box<Path> = unsafe { Box::from_raw(native_ptr as *mut _) };
        path.quad_to(Point::new(cpx, cpy), Point::new(x, y));
        Box::into_raw(path) as *mut _ as i64
    }
}

pub(crate) fn draw_image(canvas_native_ptr: c_longlong, image_array: *mut u8, image_size: size_t, original_width: c_int, original_height: c_int, dx: c_float, dy: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let image_slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(image_array, image_size) };
    let data = Data::new_copy(image_slice);
    let info = ImageInfo::new(ISize::new(original_width, original_height), ColorType::RGBA8888, AlphaType::Premul, None);
    let image_new = Image::from_raster_data(&info, &data, (original_width * 4) as usize);
    let mut canvas = surface.canvas();
    let mut paint = Paint::default();
    paint.set_blend_mode(canvas_native.fill_paint.blend_mode());
    if canvas_native.image_smoothing_enabled {
        match canvas_native.image_smoothing_quality.as_str() {
            "low" => {
                paint.set_filter_quality(FilterQuality::Low);
            }
            "medium" => {
                paint.set_filter_quality(FilterQuality::Medium);
            }
            "high" => {
                paint.set_filter_quality(FilterQuality::High);
            }
            _ => {}
        }
    } else {
        paint.set_filter_quality(FilterQuality::None);
    }

    if image_new.is_some() {
        canvas.draw_image(
            &image_new.unwrap(),
            Point::new(dx, dy),
            Some(&paint),
        );
        canvas.flush();
    }
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn draw_image_dw(canvas_native_ptr: c_longlong, image_array: *mut u8, image_size: size_t, original_width: c_int, original_height: c_int, dx: c_float, dy: c_float, d_width: c_float, d_height: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let image_slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(image_array as *mut _, image_size) };

    let data = Data::new_copy(image_slice);
    let info = ImageInfo::new(ISize::new(original_width, original_height), ColorType::RGBA8888, AlphaType::Premul, None);
    let image_new = Image::from_raster_data(&info, &data, (original_width * 4) as usize);
    if image_new.is_some() {
        let mut paint = Paint::default();
        paint.set_blend_mode(canvas_native.fill_paint.blend_mode());
        if canvas_native.image_smoothing_enabled {
            match canvas_native.image_smoothing_quality.as_str() {
                "low" => {
                    paint.set_filter_quality(FilterQuality::Low);
                }
                "medium" => {
                    paint.set_filter_quality(FilterQuality::Medium);
                }
                "high" => {
                    paint.set_filter_quality(FilterQuality::High);
                }
                _ => {}
            }
        } else {
            paint.set_filter_quality(FilterQuality::None);
        }

        canvas.draw_image_rect(
            &image_new.unwrap(),
            None,
            Rect::new(dx, dy, d_width + dx, d_height + dy),
            &paint,
        );
        canvas.flush();
    }
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn draw_image_sw(canvas_native_ptr: c_longlong, image_array: *mut u8, image_size: size_t, original_width: c_int, original_height: c_int, sx: c_float, sy: c_float, s_width: c_float, s_height: c_float, dx: c_float, dy: c_float, d_width: c_float, d_height: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let image_slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(image_array as *mut _, image_size) };

    let data = Data::new_copy(image_slice);
    let info = ImageInfo::new(ISize::new(original_width, original_height), ColorType::RGBA8888, AlphaType::Premul, None);
    let image_new = Image::from_raster_data(&info, &data, (original_width * 4) as usize);

    if image_new.is_some() {
        let src_rect = Rect::new(sx, sy, s_width + sx, s_height + sy);
        let mut paint = Paint::default();
        paint.set_blend_mode(canvas_native.fill_paint.blend_mode());
        if canvas_native.image_smoothing_enabled {
            match canvas_native.image_smoothing_quality.as_str() {
                "low" => {
                    paint.set_filter_quality(FilterQuality::Low);
                }
                "medium" => {
                    paint.set_filter_quality(FilterQuality::Medium);
                }
                "high" => {
                    paint.set_filter_quality(FilterQuality::High);
                }
                _ => {}
            }
        } else {
            paint.set_filter_quality(FilterQuality::None);
        }

        canvas.draw_image_rect(
            &image_new.unwrap(),
            Some((&src_rect, SrcRectConstraint::Strict)),
            Rect::new(dx, dy, d_width + dx, d_height + dy),
            &paint,
        );
        canvas.flush();
    }
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn draw_image_encoded(canvas_native_ptr: c_longlong, image_array: *mut u8, image_size: size_t, original_width: c_int, original_height: c_int, dx: c_float, dy: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let image_slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(image_array, image_size) };
    let data = Data::new_copy(image_slice);
    let image_new = Image::from_encoded(&data, None);
    let mut canvas = surface.canvas();
    if image_new.is_some() {
        canvas.draw_image(
            &image_new.unwrap(),
            Point::new(dx, dy),
            None,
        );
        canvas.flush();
    }
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn draw_image_dw_encoded(canvas_native_ptr: c_longlong, image_array: *mut u8, image_size: size_t, original_width: c_int, original_height: c_int, dx: c_float, dy: c_float, d_width: c_float, d_height: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let image_slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(image_array as *mut _, image_size) };

    let data = Data::new_copy(image_slice);
    let image_new = Image::from_encoded(&data, None);

    if image_new.is_some() {
        let mut paint = Paint::default();
        canvas.draw_image_rect(
            &image_new.unwrap(),
            None,
            Rect::new(dx, dy, d_width + dx, d_height + dy),
            &paint,
        );
        canvas.flush();
    }
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn draw_image_sw_encoded(canvas_native_ptr: c_longlong, image_array: *mut u8, image_size: size_t, original_width: c_int, original_height: c_int, sx: c_float, sy: c_float, s_width: c_float, s_height: c_float, dx: c_float, dy: c_float, d_width: c_float, d_height: c_float) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let image_slice: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(image_array as *mut _, image_size) };

    let data = Data::new_copy(image_slice);
    let image_new = Image::from_encoded(&data, None);


    if image_new.is_some() {
        let src_rect = Rect::new(sx, sy, s_width + sx, s_height + sy);
        let mut paint = Paint::default();
        canvas.draw_image_rect(
            &image_new.unwrap(),
            Some((&src_rect, SrcRectConstraint::Strict)),
            Rect::new(dx, dy, d_width + dx, d_height + dy),
            &paint,
        );
        canvas.flush();
    }
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn save(canvas_native_ptr: c_longlong) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    canvas.save();
    let count = canvas.save_count();
    let size = &canvas_native.font.size().clone();
    let canvas_state = CanvasState {
        stroke_paint: canvas_native.stroke_paint.clone(),
        fill_paint: canvas_native.fill_paint.clone(),
        font: Font::from_typeface(&canvas_native.font.typeface_or_default(), size.to_owned()),
        path: canvas_native.path.clone(),
        line_dash_offset: canvas_native.line_dash_offset,
        shadow_blur: canvas_native.shadow_blur,
        shadow_color: canvas_native.shadow_color,
        shadow_offset_x: canvas_native.shadow_offset_x,
        shadow_offset_y: canvas_native.shadow_offset_y,
        image_smoothing_enabled: canvas_native.image_smoothing_enabled,
        image_smoothing_quality: canvas_native.image_smoothing_quality.clone(),
        device_scale: canvas_native.device_scale,
        text_align: canvas_native.text_align.clone(),
    };


    let state = &mut canvas_native.state;
    state.push(
        CanvasStateItem::new(Box::into_raw(Box::new(canvas_state)) as *mut _ as i64, count)
    );
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn restore(canvas_native_ptr: c_longlong) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let state_item = canvas_native.state.pop();
    if state_item.is_some() {
        let item = state_item.unwrap();
        canvas.restore_to_count(item.count);
        if item.state > 0 {
            let canvas_state: Box<CanvasState> = unsafe { Box::from_raw(item.state as *mut _) };
            canvas_native.restore_from_state_box(canvas_state);
        }
    }
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn clip_rule(canvas_native_ptr: c_longlong, fill_rule: *const c_char) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let mut fill_type: FillType;
    let rule = unsafe { CStr::from_ptr(fill_rule as *mut _).to_str().unwrap_or("nonzero") };
    match rule {
        "evenodd" => { fill_type = FillType::EventOdd }
        _ => { fill_type = FillType::Winding }
    };
    let path = Path::new();
    &canvas_native.path.set_fill_type(fill_type);
    canvas.clip_path(&canvas_native.path, Some(ClipOp::Intersect), Some(true));
    &canvas_native.path.set_fill_type(path.fill_type());
    canvas.flush();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_line_cap(canvas_native_ptr: c_longlong, line_cap: *const c_char) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let cap = unsafe { CStr::from_ptr(line_cap as *mut _) }.to_str().unwrap_or("butt");
    match cap {
        "round" => {
            canvas_native.stroke_paint.set_stroke_cap(Cap::Round);
        }
        "square" => {
            canvas_native.stroke_paint.set_stroke_cap(Cap::Square);
        }
        _ => {
            canvas_native.stroke_paint.set_stroke_cap(Cap::Butt);
        }
    };
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_line_join(canvas_native_ptr: c_longlong, line_cap: *const c_char) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let cap = unsafe { CStr::from_ptr(line_cap as *mut _) }.to_str().unwrap_or("miter");
    match cap {
        "round" => {
            canvas_native.stroke_paint.set_stroke_join(Join::Round);
        }
        "bevel" => {
            canvas_native.stroke_paint.set_stroke_join(Join::Bevel);
        }
        _ => {
            canvas_native.stroke_paint.set_stroke_join(Join::Miter);
        }
    };
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_global_alpha(canvas_native_ptr: c_longlong, alpha: u8) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    canvas_native.fill_paint.set_alpha(alpha);
    canvas_native.stroke_paint.set_alpha(alpha);
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_miter_limit(canvas_native_ptr: c_longlong, limit: f32) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    canvas_native.stroke_paint.set_stroke_miter(limit);
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_line_dash_offset(canvas_native_ptr: c_longlong, offset: f32) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    canvas_native.line_dash_offset = offset;
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_shadow_blur(canvas_native_ptr: c_longlong, limit: f32) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    canvas_native.shadow_blur = limit;
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_shadow_color(canvas_native_ptr: c_longlong, color: u32) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    canvas_native.shadow_color = color;
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_shadow_offset_x(canvas_native_ptr: c_longlong, x: f32) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    canvas_native.shadow_offset_x = x;
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_shadow_offset_y(canvas_native_ptr: c_longlong, y: f32) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    canvas_native.shadow_offset_y = y;
    Box::into_raw(canvas_native) as *mut _ as i64
}


pub(crate) fn get_measure_text(canvas_native_ptr: c_longlong, text: *const c_char) -> CanvasTextMetrics {
    let mut metrics = CanvasTextMetrics { width: 0.0 };
    if canvas_native_ptr == 0 { return metrics; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let string = unsafe { CStr::from_ptr(text as *const _).to_str().unwrap_or("") };
    let measurement = canvas_native.font.measure_str(string, None);
    metrics.width = measurement.0;
    Box::into_raw(canvas_native);
    metrics
}

pub(crate) fn create_image_data(width: c_int, height: c_int) -> Vec<u8> {
    let size = (width * height) * 4;
    vec![0u8; size as usize]
}

pub(crate) fn put_image_data(canvas_native_ptr: c_longlong, data: *const u8, data_size: size_t, width: c_int, height: c_int, x: c_float, y: c_float, dirty_x: c_float, dirty_y: c_float, dirty_width: c_int, dirty_height: c_int) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let mut array = unsafe { std::slice::from_raw_parts(data, data_size) };
    let mut data = Data::new_copy(array);
    // is surface is opaque use AlphaType::Opaque
    let mut w = width;
    let mut h = height;
    if dirty_width > -1 {
        w = dirty_width;
    }
    if dirty_height > -1 {
        h = dirty_height
    }
    let mut info = ImageInfo::new(ISize::new(width, height), ColorType::RGBA8888, AlphaType::Premul, None);
    let row_bytes = (width * 4) as usize;
    let mut image = Image::from_raster_data(&info, &data, row_bytes);
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    canvas.write_pixels(&info, &array, row_bytes, IPoint::new(((x + dirty_x) as i32), ((y + dirty_y) as i32)));
    canvas.flush();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn get_image_data(canvas_native_ptr: c_longlong, sx: c_float, sy: c_float, sw: size_t, sh: size_t) -> (c_longlong, Vec<u8>) {
    let mut pixels = Vec::new();
    if canvas_native_ptr == 0 { return (0, pixels); }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    let mut info = ImageInfo::new_n32_premul(ISize::new(sw as i32, sh as i32), None);
    let row_bytes = info.width() * 4;
    let mut slice = vec![255u8; (row_bytes * info.height()) as usize];
    canvas.read_pixels(&mut info, slice.as_mut_slice(), row_bytes as usize, IPoint::new(sx as _, sy as _));
    let ptr = Box::into_raw(canvas_native) as *mut _ as i64;
    (ptr, slice)
}

pub(crate) fn free_image_data(data: *const u8) {
    Box::from(data);
}

pub(crate) fn set_image_smoothing_enabled(canvas_native_ptr: c_longlong, enabled: bool) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };

    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_image_smoothing_quality(canvas_native_ptr: c_longlong, quality: *const c_char) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };

    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn set_text_align(canvas_native_ptr: c_longlong, alignment: *const c_char) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    // TODO
    // set default alignment based on locale
    let text_alignment = unsafe { CStr::from_ptr(alignment as *const _) }.to_str().unwrap_or("left");
    canvas_native.text_align = text_alignment.to_string();
    Box::into_raw(canvas_native) as *mut _ as i64
}


pub(crate) fn reset_transform(canvas_native_ptr: c_longlong) -> c_longlong {
    if canvas_native_ptr == 0 { return 0; }
    let mut canvas_native: Box<CanvasNative> = unsafe { Box::from_raw(canvas_native_ptr as *mut _) };
    let mut surface = &mut canvas_native.surface;
    let mut canvas = surface.canvas();
    canvas.reset_matrix();
    Box::into_raw(canvas_native) as *mut _ as i64
}

pub(crate) fn add_path_to_path(path_native_ptr: c_longlong, path_to_add_native_ptr: c_longlong) -> c_longlong {
    if path_native_ptr > 0 && path_to_add_native_ptr > 0 {
        let mut path: Box<Path> = unsafe { Box::from_raw(path_native_ptr as *mut _) };
        let mut path_to_add: Box<Path> = unsafe { Box::from_raw(path_to_add_native_ptr as *mut _) };
        let matrix = Matrix::default();
        path.add_path_matrix(&path_to_add, &matrix, None);
        Box::into_raw(path_to_add);
        return Box::into_raw(path) as *mut _ as i64;
    }
    path_native_ptr
}

use std::borrow::{Cow, BorrowMut};
use quick_xml::events::Event;
use quick_xml::Reader;
use skia_safe::image::CachingHint;
use skia_safe::font::Edging;
use skia_safe::utils::text_utils::Align;


pub(crate) fn draw_svg_image(svg_canvas_native_ptr: c_longlong, svg: *const c_char) -> c_longlong {
    if svg_canvas_native_ptr == 0 { return 0; }
    let mut svg_canvas_native: Box<SVGCanvasNative> = unsafe { Box::from_raw(svg_canvas_native_ptr as *mut _) };
    let svg_surface = &mut svg_canvas_native.surface;
    let canvas = svg_surface.canvas();
    let mut rect = Rect::new_empty();
    let mut svg_canvas = Canvas::new(rect.clone());
    if !svg.is_null() {
        let svg_string = unsafe { CStr::from_ptr(svg as _) };
        let string = svg_string.to_str().unwrap_or("");
        if !string.is_empty() {
            let mut reader = Reader::from_str(string);
            let mut buf = Vec::new();
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) => {
                        match e.name() {
                            b"svg" => {
                                ;
                                let attributes = e.attributes().map(|a| a.unwrap()).collect::<Vec<_>>();
                                for attribute in attributes.iter() {
                                    let key = String::from_utf8_lossy(attribute.key).to_string();
                                    let val = attribute.unescape_and_decode_value(&reader).unwrap();
                                    match key.as_str() {
                                        "width" => {
                                            &rect.set_wh(val.parse::<f32>().unwrap(), rect.height());
                                        }
                                        "height" => {
                                            &rect.set_wh(rect.width(), val.parse::<f32>().unwrap());
                                        }
                                        _ => {}
                                    }
                                }
                                svg_canvas = Canvas::new(rect.clone());
                            }
                            b"circle" => {
                                let attributes = e.attributes().map(|a| a.unwrap()).collect::<Vec<_>>();
                                let mut path = Path::new();
                                let mut fill_paint = Paint::default();
                                fill_paint.set_style(Style::Fill);
                                let mut stroke_paint = Paint::default();
                                stroke_paint.set_style(Style::Stroke);
                                let mut point = Point::new(0.0, 0.0);
                                let mut radius = 0f32;
                                for attribute in attributes.iter() {
                                    let key = String::from_utf8_lossy(attribute.key).to_string();
                                    let val = attribute.unescape_and_decode_value(&reader).unwrap();
                                    match key.as_str() {
                                        "cx" => {
                                            point.x = val.parse::<f32>().unwrap();
                                        }
                                        "cy" => {
                                            point.y = val.parse::<f32>().unwrap();
                                        }
                                        "r" => {
                                            radius = val.parse::<f32>().unwrap();
                                        }
                                        "stroke" => {
                                            &stroke_paint.set_color(ColorParser::from_str(val.as_str()));
                                        }
                                        "stroke-width" => {
                                            &stroke_paint.set_stroke_width(val.parse::<f32>().unwrap());
                                        }
                                        "fill" => {
                                            &fill_paint.set_color(ColorParser::from_str(val.as_str()));
                                        }
                                        _ => {}
                                    }
                                }
                                path.add_circle(point, radius, None);
                                &svg_canvas.draw_path(&path, &fill_paint);
                                &svg_canvas.draw_path(&path, &stroke_paint);
                            }
                            b"text" => {}
                            _ => {}
                        }
                    }
                    Ok(Event::Empty(ref e)) => {
                        match e.name() {
                            b"circle" => {
                                let attributes = e.attributes().map(|a| a.unwrap()).collect::<Vec<_>>();
                                let mut path = Path::new();
                                let mut fill_paint = Paint::default();
                                fill_paint.set_anti_alias(true);
                                fill_paint.set_style(Style::Fill);
                                let mut stroke_paint = Paint::default();
                                stroke_paint.set_anti_alias(true);
                                stroke_paint.set_style(Style::Stroke);
                                let mut point = Point::new(0.0, 0.0);
                                let mut radius = 0f32;
                                for attribute in attributes.iter() {
                                    let key = String::from_utf8_lossy(attribute.key).to_string();
                                    let val = attribute.unescape_and_decode_value(&reader).unwrap();
                                    match key.as_str() {
                                        "cx" => {
                                            point.x = val.parse::<f32>().unwrap();
                                        }
                                        "cy" => {
                                            point.y = val.parse::<f32>().unwrap();
                                        }
                                        "r" => {
                                            radius = val.parse::<f32>().unwrap();
                                        }
                                        "stroke" => {
                                            &stroke_paint.set_color(ColorParser::from_str(val.as_str()));
                                        }
                                        "stroke-width" => {
                                            &stroke_paint.set_stroke_width(val.parse::<f32>().unwrap());
                                        }
                                        "fill" => {
                                            &fill_paint.set_color(ColorParser::from_str(val.as_str()));
                                        }
                                        _ => {}
                                    }
                                }
                                path.add_circle(point, radius, None);
                                &canvas.draw_path(&path, &fill_paint);
                                &canvas.draw_path(&path, &stroke_paint);
                            }
                            b"rect" => {
                                let attributes = e.attributes().map(|a| a.unwrap()).collect::<Vec<_>>();
                                let mut path = Path::new();
                                let mut fill_paint = Paint::default();
                                fill_paint.set_anti_alias(true);
                                fill_paint.set_style(Style::Fill);
                                let mut stroke_paint = Paint::default();
                                stroke_paint.set_anti_alias(true);
                                stroke_paint.set_style(Style::Stroke);
                                let mut rect = Rect::new_empty();
                                for attribute in attributes.iter() {
                                    let key = String::from_utf8_lossy(attribute.key).to_string();
                                    let val = attribute.unescape_and_decode_value(&reader).unwrap();
                                    match key.as_str() {
                                        "width" => {
                                            rect.right = val.parse::<f32>().unwrap();
                                        }
                                        "height" => {
                                            rect.bottom = val.parse::<f32>().unwrap();
                                        }
                                        "style" => {
                                            let mut styles = StyleParser::from_str(val.as_ref());
                                            for style in styles.iter() {
                                                match style.0 {
                                                    "width" => { rect.right = style.1.parse::<f32>().unwrap(); }
                                                    "height" => { rect.bottom = style.1.parse::<f32>().unwrap(); }
                                                    "stroke" => {
                                                        &stroke_paint.set_color(ColorParser::from_str(style.1));
                                                    }
                                                    "stroke-width" => {
                                                        &stroke_paint.set_stroke_width(style.1.parse::<f32>().unwrap());
                                                    }
                                                    "fill" => {
                                                        &fill_paint.set_color(ColorParser::from_str(style.1));
                                                    }
                                                    "stroke-opacity" => { &stroke_paint.set_alpha((style.1.parse::<f32>().unwrap_or(1.0) * 255.0) as u8); }
                                                    "fill-opacity" => { &fill_paint.set_alpha((style.1.parse::<f32>().unwrap_or(1.0) * 255.0) as u8); }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        "stroke" => {
                                            &stroke_paint.set_color(ColorParser::from_str(val.as_str()));
                                        }
                                        "stroke-width" => {
                                            &stroke_paint.set_stroke_width(val.parse::<f32>().unwrap());
                                        }
                                        "fill" => {
                                            &fill_paint.set_color(ColorParser::from_str(val.as_str()));
                                        }
                                        "stroke-opacity" => { &stroke_paint.set_alpha(val.parse::<u8>().unwrap_or(255)); }
                                        "fill-opacity" => { &fill_paint.set_alpha(val.parse::<u8>().unwrap_or(255)); }
                                        _ => {}
                                    }
                                }
                                path.add_rect(rect, None);
                                &canvas.draw_path(&path, &fill_paint);
                                &canvas.draw_path(&path, &stroke_paint);
                            }
                            b"path" => {
                                let attributes = e.attributes().map(|a| a.unwrap()).collect::<Vec<_>>();
                                let mut path = Path::new();
                                let mut fill_paint = Paint::default();
                                fill_paint.set_anti_alias(true);
                                fill_paint.set_style(Style::Fill);
                                let mut stroke_paint = Paint::default();
                                stroke_paint.set_anti_alias(true);
                                stroke_paint.set_style(Style::Stroke);
                                let mut fill = false;
                                let mut stroke = false;
                                for attribute in attributes.iter() {
                                    let key = String::from_utf8_lossy(attribute.key).to_string();
                                    let val = attribute.unescape_and_decode_value(&reader).unwrap();
                                    match key.as_str() {
                                        "d" => {
                                            path = from_svg(val.as_str()).unwrap_or(Path::new())
                                        }
                                        "stroke" => {
                                            let value = val.as_str();
                                            stroke = !value.eq("none");
                                            &stroke_paint.set_color(ColorParser::from_str(value));
                                        }
                                        "stroke-width" => {
                                            &stroke_paint.set_stroke_width(val.parse::<f32>().unwrap());
                                        }
                                        "fill" => {
                                            let value = val.as_str();
                                            fill = !value.eq("none");
                                            &fill_paint.set_color(ColorParser::from_str(value));
                                        }
                                        _ => {}
                                    }
                                }
                                if fill {
                                    &canvas.draw_path(&path, &fill_paint);
                                }

                                if stroke {
                                    &canvas.draw_path(&path, &stroke_paint);
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::Text(e)) => {
                        /* let font_native_type_face = Typeface::from_name("sans-serif", font_native_style).unwrap_or(Typeface::default());
                         let font = Font::from_typeface(&font_native_type_face, 10.0);
                         let blob = TextBlob::from_str(e.unescape_and_decode(&reader).unwrap(), &font);
                         let mut paint = Paint::default();
                         let mut point = Point::new(0.0, 0.0);
                         &canvas.draw_text_blob(&blob.unwrap(), &point, &paint);*/
                    }
                    Ok(Event::End(ref e)) => {}
                    Ok(Event::Eof) => break,
                    Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                    _ => (), //
                }
                buf.clear();
            }
        }

        canvas.flush();
    }
    Box::into_raw(svg_canvas_native) as *mut _ as i64
}

pub(crate) struct StyleParser {}

impl StyleParser {
    pub fn from_str(style: &str) -> Vec<(&str, &str)> {
        let mut values: Vec<(_, _)> = Vec::new();
        let mut styles: Vec<&str> = style.split(";").collect();
        for style in styles.iter() {
            let value = *style;
            let key_value: Vec<_> = value.split(":").collect();
            let default = "";
            let k = key_value.get(0).unwrap_or(&default).to_owned();
            let v = key_value.get(1).unwrap_or(&default).to_owned();
            values.push((k, v));
        }
        values
    }
}

pub(crate) struct ColorParser {}

impl ColorParser {
    pub fn is_color(value: &str) -> bool {
        value.starts_with("#") || value.starts_with("rgb") || value.starts_with("hsl")
    }
    pub fn from_str(color: &str) -> Color {
        let mut value = color.to_lowercase();
        if value.starts_with("#") {
            Color::BLACK
        } else if value.starts_with("rgb") {
            value = value.replace("rgba(", "");
            value = value.replace("rgb(", "");
            value = value.replace(")", "");
            let mut rgb_rgba: Vec<_> = value.split(",").collect();
            let default = "255";
            let mut r = rgb_rgba.get(0).unwrap_or(&default).parse().unwrap_or(255);
            let mut g = rgb_rgba.get(1).unwrap_or(&default).parse().unwrap_or(255);
            let mut b = rgb_rgba.get(2).unwrap_or(&default).parse().unwrap_or(255);
            let mut a = rgb_rgba.get(3).unwrap_or(&default).parse().unwrap_or(255);

            Color::from_argb(a, r, g, b)
        } else if value.starts_with("hsl") {
            Color::BLACK
        } else {
            match value.as_str() {
                "red" => Color::RED,
                "blue" => Color::BLUE,
                "green" => Color::GREEN,
                "pink" => Color::from_rgb(255, 192, 203),
                _ => Color::BLACK
            }
        }
    }
}