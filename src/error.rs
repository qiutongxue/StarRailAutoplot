use std::sync::{MutexGuard, TryLockError};

#[derive(Debug, thiserror::Error)]
pub enum SrPlotError {
    #[error("截图失败: {0}")]
    Screenshot(String),
    #[error("模拟鼠标或键盘输入发生错误：{0}")]
    Input(#[from] enigo::InputError),
    #[error("处理图片时发生错误：{0}")]
    ImageProcessing(String),
    #[error("死锁了，肯定是代码逻辑出了问题：{0}")]
    Deadlock(String),
    #[error("未知错误")]
    Unexcepted,
}

pub type SrPlotResult<T> = Result<T, SrPlotError>;

impl<T> From<TryLockError<MutexGuard<'_, T>>> for SrPlotError {
    fn from(err: TryLockError<MutexGuard<'_, T>>) -> Self {
        Self::Deadlock(err.to_string())
    }
}

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
