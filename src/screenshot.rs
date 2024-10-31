use image::{DynamicImage, RgbaImage};

use crate::{
    automation::Crop,
    error::{SrPlotError, SrPlotResult},
    utils::get_window,
};

pub fn take_screenshot(
    title: &str,
    crop: Option<(u32, u32, u32, u32)>,
) -> SrPlotResult<(RgbaImage, Crop, f64)> {
    if let Some(window) = get_window(title) {
        let mut screenshot = window
            .capture_image()
            .map_err(|e| SrPlotError::Screenshot(e.to_string()))?;

        let (real_width, real_height) = (window.width(), window.height());
        let mut screenshot_factor = 1.0;
        if real_width > 1920 {
            screenshot_factor = 1920.0 / real_width as f64;
            let (_, _, w, h) = crop.unwrap_or((0, 0, real_width, real_height));
            screenshot = DynamicImage::ImageRgba8(screenshot)
                .resize(w, h, image::imageops::FilterType::Nearest)
                .to_rgba8();
        }

        if let Some((x, y, w, h)) = crop {
            Ok((
                DynamicImage::ImageRgba8(screenshot)
                    .crop(x, y, w, h)
                    .into_rgba8(),
                (x + window.x() as u32, y + window.y() as u32, w, h),
                screenshot_factor,
            ))
        } else {
            let region = (
                window.x() as u32,
                window.y() as u32,
                window.width(),
                window.height(),
            );
            Ok((screenshot, region, screenshot_factor))
        }
    } else {
        Err(SrPlotError::Screenshot(format!("窗口「{}」不存在", title)))
    }
}
