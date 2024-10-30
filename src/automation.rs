use std::{
    collections::HashMap,
    thread,
    time::{self, Duration},
};

use image::{codecs::bmp::BmpEncoder, RgbaImage};
use opencv::core::{Mat, MatTraitConst, Size};
use opencv::{
    core::Point,
    imgcodecs::{imdecode, ImreadModes},
};

use crate::{input::Input, screenshot, utils::scale_and_match_template};

pub struct Automation {
    screenshot: Option<RgbaImage>,
    screenshot_pos: Option<(u32, u32, u32, u32)>,
    screenshot_factor: f64,
    window_title: String,
    cache: HashMap<String, Mat>,
}

pub type ScaleRange = (f64, f64);
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
    pub fn take_screenshot(&mut self, crop: Option<Crop>) {
        let timer = time::Instant::now();
        loop {
            let result = screenshot::take_screenshot(&self.window_title, crop).ok();
            if let Some((screenshot, screenshot_pos, screenshot_factor)) = result {
                self.screenshot = Some(screenshot);
                self.screenshot_pos = Some(screenshot_pos);
                self.screenshot_factor = screenshot_factor;
                log::debug!(
                    "截图成功，耗时：{}ms, 截图区域：{:?}， 缩放比例：{:.2}",
                    timer.elapsed().as_millis(),
                    screenshot_pos,
                    screenshot_factor
                );
                break;
            } else {
                log::error!("截图失败，可能未找到游戏窗口");
            }
            thread::sleep(Duration::from_secs(1));
            if timer.elapsed().as_secs() > 10 {
                log::error!("截图超时");
                break;
            }
        }
    }

    pub fn find_element(
        &mut self,
        target: (&'static str, &[u8]),
        threshold: f64,
        take_screenshot: bool,
        scale_range: Option<ScaleRange>,
        crop: Option<Crop>,
    ) -> Option<((u32, u32), (u32, u32))> {
        if take_screenshot {
            self.take_screenshot(crop);
        }
        log::debug!("scale_range: {:?}", scale_range);
        let (target_name, target) = target;
        if !self.cache.contains_key(target_name) {
            let template = imdecode(&target, ImreadModes::IMREAD_COLOR as i32).unwrap();
            self.cache.insert(target_name.to_string(), template);
        }

        let template = self.cache.get(target_name).unwrap();

        let screenshot = {
            let image = self.screenshot.as_ref().unwrap();
            let mut buffer =
                Vec::with_capacity(image.width() as usize * image.height() as usize + 1000);
            let mut encoder = BmpEncoder::new(&mut buffer);
            encoder
                .encode(
                    &image,
                    image.width(),
                    image.height(),
                    image::ExtendedColorType::Rgba8,
                )
                .unwrap();
            imdecode(&buffer.as_slice(), ImreadModes::IMREAD_COLOR as i32).unwrap()
        };

        let (match_val, match_loc) =
            scale_and_match_template(&screenshot, &template, threshold, scale_range);

        log::debug!("目标图片：{}, 相似度：{:.2}", target_name, match_val);

        if match_val.is_finite() && match_val >= threshold {
            log::debug!(
                "目标图片匹配成功，位置：({:.2}, {:.2})",
                match_loc.x,
                match_loc.y
            );
            let (top_left, bottom_right) = self.calculate_positions(&template, match_loc);
            return Some((top_left, bottom_right));
        }

        None
    }

    pub fn click_element(
        &mut self,
        target: (&'static str, &[u8]),
        threshold: f64,
        crop: Option<Crop>,
        scale_range: Option<ScaleRange>,
    ) {
        let coordinates = self.find_element(target, threshold, true, scale_range, crop);
        if let Some(coordinates) = coordinates {
            log::debug!("coordinates: {:?}", coordinates);
            self.click_element_with_pos(coordinates);
        }
    }

    fn click_element_with_pos(&self, coordinates: ((u32, u32), (u32, u32))) {
        let ((left, top), (right, bottom)) = coordinates;
        let x = (left + right) / 2;
        let y = (top + bottom) / 2;

        Input::move_and_click(x, y);
    }

    fn calculate_positions(&self, template: &Mat, max_loc: Point) -> ((u32, u32), (u32, u32)) {
        let Size { width, height } = template.size().unwrap();

        let scale_factor = self.screenshot_factor;
        let top_left = (
            (max_loc.x as f64 / scale_factor) as u32 + self.screenshot_pos.unwrap().0,
            (max_loc.y as f64 / scale_factor) as u32 + self.screenshot_pos.unwrap().1,
        );
        let bottom_right = (
            top_left.0 + (width as f64 / scale_factor) as u32,
            top_left.1 + (height as f64 / scale_factor) as u32,
        );

        (top_left, bottom_right)
    }
}

#[cfg(test)]
mod tests {

    use image::codecs::bmp::BmpEncoder;
    use opencv::{
        core::{min_max_loc, no_array, Mat, MatTraitConst, Point},
        imgcodecs::{imdecode, imread, ImreadModes},
        imgproc::{match_template_def, TemplateMatchModes},
    };

    use crate::utils::get_window;

    #[test]
    fn test() {
        if let Some(window) = get_window("崩坏：星穹铁道") {
            println!("has window");
            let screenshot = window.capture_image().unwrap();
            let mut buf = Vec::new();
            let mut encoder = BmpEncoder::new(&mut buf);
            encoder
                .encode(
                    &screenshot.as_raw(),
                    screenshot.width(),
                    screenshot.height(),
                    image::ExtendedColorType::Rgba8,
                )
                .unwrap();
            println!("buf len: {}", buf.len());
            let mat = imdecode(&buf.as_slice(), ImreadModes::IMREAD_COLOR as i32).unwrap();
            println!("mat type: {}, depth: {}", mat.typ(), mat.depth());
            println!("mat size: {:?}", mat.size());
            let template = imread("assets/start.png", ImreadModes::IMREAD_COLOR.into()).unwrap();

            println!(
                "template type: {}, depth: {}",
                template.typ(),
                template.depth()
            );
            let mut result = Mat::default();
            match_template_def(
                &mat,
                &template,
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

            println!(
                "max_val: {}, max_loc: ({}, {})",
                max_val, max_loc.x, max_loc.y
            );

            // screenshot.save("screenshot.bmp").unwrap();
            // if let Ok(mut file) = File::open("screenshot.bmp") {
            //     let mut buf = vec![];
            //     file.read_to_end(&mut buf);
            //     println!("buf len: {}", buf.len());
            //     let mat = imdecode(&buf.as_slice(), ImreadModes::IMREAD_UNCHANGED as i32).unwrap();
            //     println!("mat size: {:?}", mat.size());
            // }
            // let saved_screenshot =
            //     imread("screenshot.png", ImreadModes::IMREAD_UNCHANGED as i32).unwrap();
        }
    }
}
