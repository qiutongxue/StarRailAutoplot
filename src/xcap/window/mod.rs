mod boxed;
mod capture;
mod impl_monitor;
mod impl_window;
mod utils;
use image::RgbaImage;

use error::XCapResult;
use impl_window::ImplWindow;

use super::error;

#[derive(Debug, Clone)]
pub struct Window {
    pub(crate) impl_window: ImplWindow,
}

impl Window {
    pub(crate) fn new(impl_window: ImplWindow) -> Window {
        Window { impl_window }
    }
}

impl Window {
    pub fn all() -> XCapResult<Vec<Window>> {
        let windows = ImplWindow::all()?
            .iter()
            .map(|impl_window| Window::new(impl_window.clone()))
            .collect();

        Ok(windows)
    }
}

impl Window {
    /// The window title
    pub fn title(&self) -> &str {
        &self.impl_window.title
    }

    /// The window x coordinate.
    pub fn x(&self) -> i32 {
        self.impl_window.x
    }
    /// The window y coordinate.
    pub fn y(&self) -> i32 {
        self.impl_window.y
    }
    /// The window pixel width.
    pub fn width(&self) -> u32 {
        self.impl_window.width
    }
    /// The window pixel height.
    pub fn height(&self) -> u32 {
        self.impl_window.height
    }
    pub fn is_active(&self) -> bool {
        self.impl_window.is_active
    }
}

impl Window {
    pub fn capture_image(&self) -> XCapResult<RgbaImage> {
        self.impl_window.capture_image()
    }
}
