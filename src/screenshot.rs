use image::{DynamicImage, RgbaImage};

use crate::{
    automation::Region,
    error::{SrPlotError, SrPlotResult},
    plot::CropRatio,
    utils::get_window,
};

pub fn take_screenshot(
    title: &str,
    crop: Option<CropRatio>,
) -> SrPlotResult<(RgbaImage, Region, f64, Region)> {
    if let Some(window) = get_window(title) {
        let mut screenshot = window
            .capture_image()
            .map_err(|e| SrPlotError::Screenshot(e.to_string()))?;

        let window_region = Region::new(
            window.x() as u32,
            window.y() as u32,
            window.width(),
            window.height(),
        );

        // 先裁剪
        if let Some(Region {
            x,
            y,
            width,
            height,
        }) = crop.map(|crop| transform_crop(crop, window_region.width, window_region.height))
        {
            screenshot = DynamicImage::ImageRgba8(screenshot)
                .crop(x, y, width, height)
                .to_rgba8();
        }

        let mut screenshot_factor = 1.0;
        // 分辨率过高，调整图片大小到 1080p，加速计算
        if window_region.width > 1920 {
            screenshot_factor = 1920.0 / window_region.width as f64;

            screenshot = DynamicImage::ImageRgba8(screenshot)
                .resize(
                    (1920.0 * crop.map_or(1.0, |c| c.2)) as u32,
                    (1080.0 * crop.map_or(1.0, |c| c.3)) as u32,
                    image::imageops::FilterType::Nearest,
                )
                .to_rgba8();
        }

        Ok((
            screenshot,
            crop.map_or(window_region, |crop| {
                let mut region = transform_crop(crop, window_region.width, window_region.height);
                region.x += window_region.x;
                region.y += window_region.y;
                region
            }),
            screenshot_factor,
            window_region,
        ))
    } else {
        Err(SrPlotError::Screenshot(format!("窗口「{}」不存在", title)))
    }
}

pub fn transform_crop(crop: CropRatio, w: u32, h: u32) -> Region {
    Region::new(
        (crop.0 * w as f32) as u32,
        (crop.1 * h as f32) as u32,
        (crop.2 * w as f32) as u32,
        (crop.3 * h as f32) as u32,
    )
}
