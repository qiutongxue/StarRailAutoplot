use opencv::{
    core::{min_max_loc, no_array, Mat, MatTraitConst, Point, Size},
    imgproc::{match_template_def, resize, InterpolationFlags, TemplateMatchModes},
};
use xcap::Window;

use crate::{automation::Crop, xcap};

pub fn get_window(title: &str) -> Option<Window> {
    Window::all().ok().and_then(|windows| {
        windows
            .into_iter()
            .find(|window| window.title().contains(title))
    })
}

pub fn transform_crop(crop: (f32, f32, f32, f32), w: u32, h: u32) -> Crop {
    (
        (crop.0 * w as f32) as u32,
        (crop.1 * h as f32) as u32,
        (crop.2 * w as f32) as u32,
        (crop.3 * h as f32) as u32,
    )
}

pub fn scale_and_match_template(
    screenshot: &Mat,
    template: &Mat,
    threshold: f64,
    scale_range: Option<(f64, f64)>,
) -> (f64, Point) {
    let mut result = Mat::default();
    log::debug!(
        "screenshot size: {:?}, template size: {:?}",
        screenshot.size(),
        template.size()
    );
    match_template_def(
        screenshot,
        template,
        &mut result,
        TemplateMatchModes::TM_CCOEFF_NORMED as i32,
    )
    .unwrap();
    let mut max_val = 0f64;
    let mut max_loc = Point::default();
    min_max_loc(
        &result,
        None,
        Some(&mut max_val),
        None,
        Some(&mut max_loc),
        &no_array(),
    )
    .unwrap();

    if scale_range.is_some() && (max_val.is_infinite() || max_val < threshold) {
        let (scale_start, scale_end) = scale_range.unwrap();
        let mut scale = scale_start;
        while scale < scale_end + 0.0001 {
            let mut scaled_template = Mat::default();
            resize(
                template,
                &mut scaled_template,
                Size::default(),
                scale,
                scale,
                InterpolationFlags::INTER_AREA as i32,
            )
            .unwrap();
            let mut result = Mat::default();
            match_template_def(
                &screenshot,
                &scaled_template,
                &mut result,
                TemplateMatchModes::TM_CCOEFF_NORMED as i32,
            )
            .unwrap();

            let mut local_max_val = 0f64;
            let mut local_max_loc = Point::default();
            min_max_loc(
                &result,
                None,
                Some(&mut local_max_val),
                None,
                Some(&mut local_max_loc),
                &no_array(),
            )
            .unwrap();
            if local_max_val > max_val {
                max_val = local_max_val;
                max_loc = local_max_loc;
            }
            scale += 0.05;
        }
    }
    (max_val, max_loc)
}
