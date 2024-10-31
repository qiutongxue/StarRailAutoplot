use std::{sync::OnceLock, thread, time::Duration};

use enigo::{Button, Direction, Enigo, Mouse, Settings};

use crate::error::SrPlotResult;

static mut ENIGO_INS: OnceLock<Enigo> = OnceLock::new();

fn get_enigo<'a>() -> &'a mut Enigo {
    unsafe {
        ENIGO_INS.get_or_init(|| Enigo::new(&Settings::default()).unwrap());
        ENIGO_INS.get_mut().unwrap()
    }
}

pub struct Input;

impl Input {
    pub fn click() -> SrPlotResult<()> {
        get_enigo().button(Button::Left, Direction::Press)?;
        // 经测试，正常的间隔大概在 75~100ms 左右
        thread::sleep(Duration::from_millis(50));
        get_enigo().button(Button::Left, Direction::Release)?;
        Ok(())
    }

    pub fn position() -> (u32, u32) {
        get_enigo()
            .location()
            .map_or((0, 0), |loc| (loc.0 as u32, loc.1 as u32))
    }

    pub fn move_mouse(x: u32, y: u32) -> SrPlotResult<()> {
        get_enigo().move_mouse(x as i32, y as i32, enigo::Coordinate::Abs)?;
        Ok(())
    }
}
