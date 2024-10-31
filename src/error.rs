// use std::sync::{MutexGuard, TryLockError};

#[derive(Debug, thiserror::Error)]
pub enum SrPlotError {
    #[error("截图失败: {0}")]
    Screenshot(String),
    #[error("模拟鼠标或键盘输入发生错误：{0}")]
    Input(#[from] enigo::InputError),
    #[error("处理图片时发生错误：{0}")]
    ImageProcessing(String),
    #[error("{0}")]
    User(String),
    #[error("未知错误")]
    Unexcepted,
}

pub type SrPlotResult<T> = Result<T, SrPlotError>;

impl From<opencv::Error> for SrPlotError {
    fn from(value: opencv::Error) -> Self {
        Self::ImageProcessing(value.to_string())
    }
}

impl From<image::ImageError> for SrPlotError {
    fn from(value: image::ImageError) -> Self {
        Self::ImageProcessing(value.to_string())
    }
}
