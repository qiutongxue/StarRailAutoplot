use std::{
    sync::{Arc, Mutex},
    thread,
    time::{self, Duration},
};

use crate::{
    automation::Automation,
    error::{SrPlotError, SrPlotResult},
    input::Input,
    utils::{get_window, transform_crop},
};

use colored::Colorize;

pub type ImageFile = (&'static str, Vec<u8>);

pub struct Plot {
    select_img: ImageFile,
    game_title_name: String,
    start_img: Vec<ImageFile>,
    is_clicking: bool,
    is_window_active: bool,
    region: Option<(u32, u32, u32, u32)>,
    auto: Arc<Mutex<Automation>>,
}

impl Plot {
    pub fn new(
        game_title_name: String,
        select_img: (&'static str, Vec<u8>),
        start_img: Vec<(&'static str, Vec<u8>)>,
    ) -> Self {
        Self {
            auto: Arc::new(Mutex::new(Automation::new(&game_title_name))),
            select_img,
            game_title_name,
            start_img,
            is_clicking: false,
            is_window_active: false,
            region: None,
        }
    }

    pub fn run(self) {
        thread::spawn(|| {
            let arc_self = Arc::new(Mutex::new(self));
            loop {
                if let Err(e) = Self::check_game_status(arc_self.clone()) {
                    log::error!("{}", format!("发生错误：\n\t {}", e).red().bold());
                    // 需要停止点击
                    arc_self.try_lock().unwrap().stop_clicking();
                }
                thread::sleep(Duration::from_millis(500));
            }
        })
        .join()
        .unwrap();
    }

    fn check_game_status(arc_self: Arc<Mutex<Self>>) -> SrPlotResult<()> {
        let time = time::Instant::now();
        // 直接取得所有权，防止锁的生命周期过长
        let game_title_name = arc_self.try_lock()?.game_title_name.clone();
        if let Some(window) = get_window(&game_title_name) {
            if window.is_active() {
                {
                    let mut lock = arc_self.try_lock()?;
                    if !lock.is_window_active {
                        lock.is_window_active = true;
                        log::info!("{}", "游戏窗口已激活！正在执行中……".green().bold());
                    }
                }
                let (width, height) = (window.width(), window.height());
                let (x, y) = (window.x() as u32, window.y() as u32);

                // 记录窗口位置，点击的时候要用
                arc_self.try_lock()?.region = Some((x, y, width, height));

                let auto = arc_self.try_lock()?.auto.clone();
                let mut auto = auto.try_lock()?;
                auto.take_screenshot(Some(transform_crop(
                    (122.0 / 1920.0, 31.0 / 1080.0, 98.0 / 1920.0, 58.0 / 1080.0),
                    width,
                    height,
                )));

                // 缩放大小，匹配窗口分辨率
                let scale = width as f64 / 1920.0;
                let scale_range = (
                    ((scale - 0.05) * 10.0).round() / 10.0,
                    ((scale + 0.05) * 10.0).round() / 10.0,
                );
                let mut should_click = false;

                for img in &arc_self.try_lock()?.start_img {
                    let result =
                        auto.find_element((img.0, &img.1), 0.9, false, Some(scale_range), None)?;
                    if result.is_some() {
                        should_click = true;
                        break;
                    }
                }

                if should_click {
                    let select_img = arc_self.try_lock()?.select_img.clone();

                    Self::start_clicking(arc_self.clone())?;
                    let _ = auto.click_element(
                        (select_img.0, &select_img.1),
                        0.9,
                        Some(transform_crop(
                            (
                                1290.0 / 1920.0,
                                442.0 / 1080.0,
                                74.0 / 1920.0,
                                400.0 / 1080.0,
                            ),
                            width,
                            height,
                        )),
                        Some(scale_range),
                    );
                }
                log::debug!("执行完毕！总耗时：{}ms", time.elapsed().as_millis());
            } else {
                let mut lock = arc_self.try_lock()?;
                if lock.is_window_active {
                    lock.is_window_active = false;
                    log::warn!("{}", "检测到游戏窗口未激活，停止执行！".blue().bold());
                }
            }
        }

        arc_self.try_lock()?.stop_clicking();
        Ok(())
    }

    fn start_clicking(arc_self: Arc<Mutex<Self>>) -> SrPlotResult<()> {
        arc_self.try_lock()?.is_clicking = true;
        let arc_self = arc_self.clone();
        thread::spawn(|| Self::click(arc_self));
        Ok(())
    }

    fn stop_clicking(&mut self) {
        self.is_clicking = false;
    }

    fn click(arc_self: Arc<Mutex<Self>>) -> SrPlotResult<()> {
        loop {
            if !arc_self.try_lock()?.is_clicking {
                break;
            }
            let (mouse_x, mouse_y) = Input::position();
            log::debug!("鼠标位置：({}, {})", mouse_x, mouse_y);
            let (x, y, w, h) = arc_self.try_lock()?.region.ok_or(SrPlotError::Unexcepted)?;
            log::debug!("窗口位置：({}, {}, {}, {})", x, y, w, h);
            if x <= mouse_x && mouse_x <= x + w && y <= mouse_y && mouse_y <= y + h {
                Input::click()?;
            } else {
                log::warn!("{}", "鼠标不在窗口内！".bold());
            }
            log::debug!("点击后等待 100ms");
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}
