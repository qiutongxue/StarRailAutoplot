use core::panic;

use image::{DynamicImage, RgbaImage};

use crate::{automation::Crop, utils::get_window};

pub fn take_screenshot(
    title: &str,
    crop: Option<(u32, u32, u32, u32)>,
) -> Result<(RgbaImage, Crop, f64), Box<dyn std::error::Error>> {
    if let Some(window) = get_window(title) {
        let mut screenshot = window.capture_image()?;
        let (real_width, real_height) = (window.width(), window.height());
        let mut screenshot_factor = 1.0;
        if real_width > 1920 {
            screenshot_factor = 1920.0 / real_width as f64;
            let (_, _, w, h) = crop.unwrap_or((0, 0, real_width, real_height));
            screenshot = DynamicImage::ImageRgba8(screenshot)
                .resize(w, h, image::imageops::FilterType::CatmullRom)
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
        panic!("Window not found: {}", title);
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::transform_crop;

    use super::take_screenshot;

    #[test]
    fn test() {
        let img = take_screenshot(
            "崩坏：星穹铁道",
            Some(transform_crop(
                (122.0 / 1920.0, 31.0 / 1080.0, 98.0 / 1920.0, 58.0 / 1080.0),
                1600,
                900,
            )),
        )
        .unwrap();
        img.0.save("test.bmp").unwrap();
    }
}
