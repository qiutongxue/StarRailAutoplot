use std::collections::HashMap;
use std::time::Instant;

use image::codecs::bmp::BmpEncoder;
use image::{ImageBuffer, Rgba};
use opencv::core::{min_max_loc, no_array, Mat, MatTraitConst, Size};

use opencv::imgproc::{match_template_def, resize, InterpolationFlags, TemplateMatchModes};
use opencv::{
    core::Point,
    imgcodecs::{imdecode, ImreadModes},
};

use crate::plot::CropRatio;
use crate::{
    error::{SrPlotError, SrPlotResult},
    input::Input,
    screenshot,
};

pub type ScaleRange = (f64, f64);
pub type Coordinate = ((u32, u32), (u32, u32));

#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

pub struct Automation {
    screenshot_mat: Option<Mat>,
    screenshot_pos: Option<Region>,
    screenshot_factor: f64,
    window_title: String,
    window_region: Option<Region>,
    cache: HashMap<String, Mat>,
}

impl Automation {
    pub fn new(window_title: &str) -> Self {
        Self {
            screenshot_mat: None,
            screenshot_pos: None,
            screenshot_factor: 1.0,
            window_region: None,
            cache: HashMap::new(),
            window_title: window_title.to_owned(),
        }
    }

    pub fn take_screenshot(&mut self, crop: Option<CropRatio>) -> SrPlotResult<()> {
        let timer = Instant::now();

        let (screenshot, screenshot_pos, screenshot_factor, window_region) =
            screenshot::take_screenshot(&self.window_title, crop)?;
        self.screenshot_pos = Some(screenshot_pos);
        self.window_region = Some(window_region);
        self.screenshot_factor = screenshot_factor;
        self.screenshot_mat = {
            let image = encode_image(&screenshot)?;
            Some(imdecode(
                &image.as_slice(),
                ImreadModes::IMREAD_COLOR as i32,
            )?)
        };

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
        target: &(&'static str, Vec<u8>),
        threshold: f64,
        scale_range: Option<ScaleRange>,
    ) -> SrPlotResult<Option<Coordinate>> {
        log::debug!("scale_range: {:?}", scale_range);
        let (target_name, target_data) = target;

        let template = {
            if !self.cache.contains_key(*target_name) {
                let template = imdecode(&target_data.as_slice(), ImreadModes::IMREAD_COLOR as i32)?;
                self.cache.insert(target_name.to_string(), template);
            }
            self.cache
                .get(*target_name)
                .ok_or(SrPlotError::Unexcepted)?
        };

        let screenshot = self
            .screenshot_mat
            .as_ref()
            .ok_or(SrPlotError::Unexcepted)?;

        let (match_val, match_loc) =
            scale_and_match_template(screenshot, template, threshold, scale_range)?;

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

    fn calculate_positions(&self, template: &Mat, max_loc: Point) -> SrPlotResult<Coordinate> {
        let Size { width, height } = template.size()?;

        let scale_factor = self.screenshot_factor;
        let Region {
            x: sspos_x,
            y: sspos_y,
            ..
        } = self.screenshot_pos.ok_or(SrPlotError::Unexcepted)?;
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

    pub fn click(&self) -> SrPlotResult<()> {
        let (mouse_x, mouse_y) = Input::position();
        log::debug!("鼠标位置：({}, {})", mouse_x, mouse_y);
        let Region {
            x,
            y,
            width,
            height,
        } = self.window_region.ok_or(SrPlotError::Unexcepted)?;
        log::debug!("窗口位置：({}, {}, {}, {})", x, y, width, height);
        if x <= mouse_x && mouse_x <= x + width && y <= mouse_y && mouse_y <= y + height {
            Input::click()
        } else {
            Err(SrPlotError::User("鼠标不在游戏窗口内！".to_string()))
        }
    }

    pub fn click_with_coordinate(&self, coordinate: Coordinate) -> SrPlotResult<()> {
        let ((left, top), (right, bottom)) = coordinate;
        let x = (left + right) / 2;
        let y = (top + bottom) / 2;

        Input::move_mouse(x, y)?;
        self.click()
    }
}

fn scale_and_match_template(
    screenshot: &Mat,
    template: &Mat,
    threshold: f64,
    scale_range: Option<(f64, f64)>,
) -> SrPlotResult<(f64, Point)> {
    log::debug!(
        "screenshot size: {:?}, template size: {:?}",
        screenshot.size(),
        template.size()
    );

    let result = match_template(screenshot, template, TemplateMatchModes::TM_CCOEFF_NORMED)?;

    let (max_val, max_loc) = match scale_range {
        Some((scale_start, scale_end)) => {
            let (mut max_val, mut max_loc) = (0f64, Point::default());
            let mut scale_factor = scale_start;
            while scale_factor < scale_end + 0.0001 && max_val < threshold {
                let scaled_template = resize_template(template, scale_factor)?;
                let result = match_template(
                    screenshot,
                    &scaled_template,
                    TemplateMatchModes::TM_CCOEFF_NORMED,
                )?;

                let (local_max_val, local_max_loc) = find_max_location(&result)?;
                if local_max_val > max_val {
                    max_val = local_max_val;
                    max_loc = local_max_loc;
                }
                scale_factor += 0.05;
            }
            (max_val, max_loc)
        }
        None => find_max_location(&result)?,
    };

    Ok((max_val, max_loc))
}

fn resize_template(template: &Mat, scale_factor: f64) -> SrPlotResult<Mat> {
    let mut scaled_template = Mat::default();
    resize(
        template,
        &mut scaled_template,
        Size::default(),
        scale_factor,
        scale_factor,
        InterpolationFlags::INTER_AREA as i32,
    )?;
    Ok(scaled_template)
}

fn find_max_location(src: &Mat) -> SrPlotResult<(f64, Point)> {
    let mut max_val = 0f64;
    let mut max_loc = Point::default();
    min_max_loc(
        src,
        None,
        Some(&mut max_val),
        None,
        Some(&mut max_loc),
        &no_array(),
    )?;
    Ok((max_val, max_loc))
}

fn match_template(image: &Mat, templ: &Mat, method: TemplateMatchModes) -> SrPlotResult<Mat> {
    let mut result = Mat::default();
    match_template_def(image, templ, &mut result, method as i32).unwrap();
    Ok(result)
}

fn encode_image(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> SrPlotResult<Vec<u8>> {
    let mut buffer = Vec::with_capacity(image.width() as usize * image.height() as usize + 1000);
    let mut encoder = BmpEncoder::new(&mut buffer);
    encoder.encode(
        image,
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(buffer)
}
