use std::{
    sync::{Arc, Mutex},
    thread,
    time::{self, Duration},
};

use crate::{
    automation::Automation,
    input::Input,
    utils::{get_window, transform_crop},
};

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
        thread::spawn(|| self.check_game_status()).join().unwrap();
    }

    fn check_game_status(self) {
        let arc_self = Arc::new(Mutex::new(self));
        loop {
            let time = time::Instant::now();
            // 直接取得所有权，防止锁的生命周期过长
            let game_title_name = arc_self.try_lock().unwrap().game_title_name.clone();
            if let Some(window) = get_window(&game_title_name) {
                if window.is_active() {
                    {
                        let mut lock = arc_self.try_lock().unwrap();
                        if !lock.is_window_active {
                            lock.is_window_active = true;
                            log::info!("游戏窗口已激活！正在执行中……");
                        }
                    }
                    let (width, height) = (window.width(), window.height());
                    let (x, y) = (window.x() as u32, window.y() as u32);

                    // 记录窗口位置，点击的时候要用
                    arc_self.try_lock().unwrap().region = Some((x, y, width, height));

                    let auto = arc_self.try_lock().unwrap().auto.clone();
                    let mut auto = auto.lock().unwrap();
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

                    for img in &arc_self.try_lock().unwrap().start_img {
                        let result =
                            auto.find_element((img.0, &img.1), 0.9, false, Some(scale_range), None);
                        if result.is_some() {
                            should_click = true;
                            break;
                        }
                    }

                    if should_click {
                        let select_img = arc_self.try_lock().unwrap().select_img.clone();

                        Self::start_clicking(arc_self.clone());
                        auto.click_element(
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
                    let mut lock = arc_self.try_lock().unwrap();
                    if lock.is_window_active {
                        lock.is_window_active = false;
                        log::warn!("检测到游戏窗口未激活，停止执行！");
                    }
                }
            }

            arc_self.try_lock().unwrap().stop_clicking();
            thread::sleep(Duration::from_millis(500));
        }
    }

    fn start_clicking(arc_self: Arc<Mutex<Self>>) {
        arc_self.try_lock().unwrap().is_clicking = true;
        let arc_self = arc_self.clone();
        thread::spawn(|| Self::click(arc_self));
    }

    fn stop_clicking(&mut self) {
        self.is_clicking = false;
    }

    fn click(arc_self: Arc<Mutex<Self>>) {
        loop {
            if !arc_self.try_lock().unwrap().is_clicking {
                break;
            }
            let (mouse_x, mouse_y) = Input::position();
            log::debug!("鼠标位置：({}, {})", mouse_x, mouse_y);
            let (x, y, w, h) = arc_self.try_lock().unwrap().region.unwrap();
            log::debug!("窗口位置：({}, {}, {}, {})", x, y, w, h);
            if x <= mouse_x && mouse_x <= x + w && y <= mouse_y && mouse_y <= y + h {
                Input::click();
            } else {
                log::warn!("鼠标不在窗口内！");
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

#[cfg(test)]
mod tests {

    // use simple_logger::SimpleLogger;

    // use super::*;

    #[test]
    fn it_works() {
        // SimpleLogger::new()
        //     .with_level(log::LevelFilter::Debug)
        //     .init()
        //     .unwrap();
        // let plot = Plot::new(
        //     "崩坏：星穹铁道".to_string(),
        //     include_bytes!("../assets/select.png").to_vec(),
        //     vec![include_bytes!("../assets/start.png").to_vec()],
        // );

        // plot.run()

        // thread::sleep(Duration::from_secs(10));
    }
}
