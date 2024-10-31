use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{automation::Automation, error::SrPlotResult, utils::get_window};

use colored::Colorize;

pub type ImageFile = (&'static str, Vec<u8>);
pub type CropRatio = (f32, f32, f32, f32);

const START_IMAGE_CROP: CropRatio = (122.0 / 1920.0, 31.0 / 1080.0, 98.0 / 1920.0, 58.0 / 1080.0);
const SELECT_IMAGE_CROP: CropRatio = (
    1290.0 / 1920.0,
    442.0 / 1080.0,
    74.0 / 1920.0,
    400.0 / 1080.0,
);

pub struct Plot {
    select_img: ImageFile,
    game_title_name: String,
    start_img: Vec<ImageFile>,
    is_window_active: bool,
    is_window_exist: bool,
    auto: Automation,
}

impl Plot {
    pub fn new(
        game_title_name: String,
        select_img: (&'static str, Vec<u8>),
        start_img: Vec<(&'static str, Vec<u8>)>,
    ) -> Self {
        Self {
            auto: Automation::new(&game_title_name),
            select_img,
            game_title_name,
            start_img,
            is_window_active: false,
            is_window_exist: false,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Err(e) = self.check_game_status() {
                log::error!("{}", format!("{}", e).red().bold());
            }
            thread::sleep(Duration::from_millis(500));
        }
    }

    fn check_game_status(&mut self) -> SrPlotResult<()> {
        let time = Instant::now();
        if let Some(window) = get_window(&self.game_title_name) {
            self.is_window_exist = true;
            if window.is_active() {
                handle_status_change(&mut self.is_window_active, true, || {
                    log::info!("{}", "游戏窗口已激活！正在执行中……".green().bold())
                });

                self.auto.take_screenshot(START_IMAGE_CROP.into())?;

                // 缩放大小，匹配窗口分辨率
                let scale_factor = window.width() as f64 / 1920.0;
                let scale_range = (scale_factor < 1.0).then(|| {
                    (
                        ((scale_factor - 0.05) * 10.0).round() / 10.0,
                        ((scale_factor + 0.05) * 10.0).round() / 10.0,
                    )
                });

                for img in &self.start_img {
                    let result = self.auto.find_element((img.0, &img.1), 0.9, scale_range)?;
                    if result.is_some() {
                        let select_img = &self.select_img;
                        // 查找是否有选项要点
                        self.auto.take_screenshot(SELECT_IMAGE_CROP.into())?;
                        match self.auto.find_element(
                            (select_img.0, &select_img.1),
                            0.88, // 遇到过 0.89 匹配不上，所以降低 threshold
                            scale_range,
                        )? {
                            // 有选项就点击选项
                            Some(coordinate) => self.auto.click_with_coordinate(coordinate)?,
                            // 没选项就随便点
                            None => self.auto.click()?,
                        }
                        break;
                    }
                }
                log::debug!("执行完毕！总耗时：{}ms", time.elapsed().as_millis());
            } else {
                handle_status_change(&mut self.is_window_active, false, || {
                    log::warn!("{}", "检测到游戏窗口未激活，停止执行！".blue().bold())
                });
            }
        } else {
            self.is_window_active = false;
            handle_status_change(&mut self.is_window_exist, false, || {
                log::warn!("{}", "未检测到游戏窗口，等待游戏启动……".cyan().bold())
            });
        }
        Ok(())
    }
}

fn handle_status_change<T: Eq>(status: &mut T, target: T, event: impl FnOnce()) {
    if *status != target {
        *status = target;
        event();
    }
}
