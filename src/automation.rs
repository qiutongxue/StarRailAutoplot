use std::collections::HashMap;
use std::time::Instant;

use image::{codecs::bmp::BmpEncoder, RgbaImage};
use opencv::core::{min_max_loc, no_array, Mat, MatTraitConst, Size};

use opencv::imgproc::{match_template_def, resize, InterpolationFlags, TemplateMatchModes};
use opencv::{
    core::Point,
    imgcodecs::{imdecode, ImreadModes},
};

use crate::{
    error::{SrPlotError, SrPlotResult},
    input::Input,
    screenshot,
};

pub struct Automation {
    screenshot: Option<RgbaImage>,
    screenshot_pos: Option<(u32, u32, u32, u32)>,
    screenshot_factor: f64,
    window_title: String,
    cache: HashMap<String, Mat>,
}

pub type ScaleRange = (f64, f64);
type Coordinate = ((u32, u32), (u32, u32));
pub type Crop = (u32, u32, u32, u32);

impl Automation {
    pub fn new(window_title: &str) -> Self {
        Self {
            screenshot: None,
            screenshot_pos: None,
            screenshot_factor: 1.0,
            cache: HashMap::new(),
            window_title: window_title.to_owned(),
        }
    }

    pub fn take_screenshot(&mut self, crop: Option<Crop>) -> SrPlotResult<()> {
        let timer = Instant::now();

        let (screenshot, screenshot_pos, screenshot_factor) =
            screenshot::take_screenshot(&self.window_title, crop)?;
        self.screenshot = Some(screenshot);
        self.screenshot_pos = Some(screenshot_pos);
        self.screenshot_factor = screenshot_factor;
        log::debug!(
            "截图成功，耗时：{}ms, 截图区域：{:?}，缩放比例：{:.2}",
            timer.elapsed().as_millis(),
            screenshot_pos,
            screenshot_factor
        );
        Ok(())
    }

    pub fn find_element(
        &mut self,
        target: (&'static str, &[u8]),
        threshold: f64,
        scale_range: ScaleRange,
    ) -> SrPlotResult<Option<Coordinate>> {
        log::debug!("scale_range: {:?}", scale_range);
        let (target_name, target) = target;
        if !self.cache.contains_key(target_name) {
            let template = imdecode(&target, ImreadModes::IMREAD_COLOR as i32)?;
            self.cache.insert(target_name.to_string(), template);
        }

        let template = self.cache.get(target_name).ok_or(SrPlotError::Unexcepted)?;

        let screenshot = {
            let image = self.screenshot.as_ref().ok_or(SrPlotError::Unexcepted)?;
            let mut buffer =
                Vec::with_capacity(image.width() as usize * image.height() as usize + 1000);
            let mut encoder = BmpEncoder::new(&mut buffer);
            encoder.encode(
                image,
                image.width(),
                image.height(),
                image::ExtendedColorType::Rgba8,
            )?;
            imdecode(&buffer.as_slice(), ImreadModes::IMREAD_COLOR as i32)?
        };

        let (match_val, match_loc) =
            scale_and_match_template(&screenshot, template, threshold, scale_range.into())?;

        log::debug!("目标图片：{}, 相似度：{:.2}", target_name, match_val);

        if match_val.is_finite() && match_val >= threshold {
            log::debug!(
                "目标图片匹配成功，位置：({:.2}, {:.2})",
                match_loc.x,
                match_loc.y
            );
            let (top_left, bottom_right) = self.calculate_positions(template, match_loc)?;
            Ok(Some((top_left, bottom_right)))
        } else {
            Ok(None)
        }
    }

    pub fn click_element(
        &mut self,
        target: (&'static str, &[u8]),
        threshold: f64,
        scale_range: ScaleRange,
        crop: Crop,
    ) -> SrPlotResult<Option<()>> {
        self.take_screenshot(crop.into())?;
        let coordinates = self.find_element(target, threshold, scale_range)?;
        if let Some(coordinates) = coordinates {
            log::debug!("coordinates: {:?}", coordinates);
            self.click_element_with_pos(coordinates)?;
            return Ok(Some(()));
        }
        Ok(None)
    }

    fn click_element_with_pos(&self, coordinates: Coordinate) -> SrPlotResult<()> {
        let ((left, top), (right, bottom)) = coordinates;
        let x = (left + right) / 2;
        let y = (top + bottom) / 2;

        Input::move_and_click(x, y)?;
        Ok(())
    }

    fn calculate_positions(&self, template: &Mat, max_loc: Point) -> SrPlotResult<Coordinate> {
        let Size { width, height } = template.size()?;

        let scale_factor = self.screenshot_factor;
        let (sspos_x, sspos_y, _, _) = self.screenshot_pos.ok_or(SrPlotError::Unexcepted)?;
        let top_left = (
            (max_loc.x as f64 / scale_factor) as u32 + sspos_x,
            (max_loc.y as f64 / scale_factor) as u32 + sspos_y,
        );
        let bottom_right = (
            top_left.0 + (width as f64 / scale_factor) as u32,
            top_left.1 + (height as f64 / scale_factor) as u32,
        );

        Ok((top_left, bottom_right))
    }
}

pub fn scale_and_match_template(
    screenshot: &Mat,
    template: &Mat,
    threshold: f64,
    scale_range: Option<(f64, f64)>,
) -> SrPlotResult<(f64, Point)> {
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
    )?;
    let mut max_val = 0f64;
    let mut max_loc = Point::default();
    min_max_loc(
        &result,
        None,
        Some(&mut max_val),
        None,
        Some(&mut max_loc),
        &no_array(),
    )?;
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
            )?;
            let mut result = Mat::default();
            match_template_def(
                &screenshot,
                &scaled_template,
                &mut result,
                TemplateMatchModes::TM_CCOEFF_NORMED as i32,
            )?;

            let mut local_max_val = 0f64;
            let mut local_max_loc = Point::default();
            min_max_loc(
                &result,
                None,
                Some(&mut local_max_val),
                None,
                Some(&mut local_max_loc),
                &no_array(),
            )?;
            if local_max_val > max_val {
                max_val = local_max_val;
                max_loc = local_max_loc;
            }
            scale += 0.05;
        }
    }
    Ok((max_val, max_loc))
}
