use std::mem;
use windows::Win32::Graphics::Gdi::{
    GetDeviceCaps, GetMonitorInfoW, DESKTOPHORZRES, HMONITOR, HORZRES, MONITORINFO, MONITORINFOEXW,
};

use crate::xcap::error::XCapResult;

use super::boxed::BoxHDC;

// A 函数与 W 函数区别
// https://learn.microsoft.com/zh-cn/windows/win32/learnwin32/working-with-strings

#[derive(Debug, Clone)]
pub(crate) struct ImplMonitor {
    #[allow(unused)]
    pub hmonitor: HMONITOR,
    #[allow(unused)]
    pub monitor_info_ex_w: MONITORINFOEXW,
    pub scale_factor: f32,
}

impl ImplMonitor {
    pub fn new(hmonitor: HMONITOR) -> XCapResult<ImplMonitor> {
        let mut monitor_info_ex_w = MONITORINFOEXW::default();
        monitor_info_ex_w.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
        let monitor_info_ex_w_ptr =
            &mut monitor_info_ex_w as *mut MONITORINFOEXW as *mut MONITORINFO;

        // https://learn.microsoft.com/zh-cn/windows/win32/api/winuser/nf-winuser-getmonitorinfoa
        unsafe { GetMonitorInfoW(hmonitor, monitor_info_ex_w_ptr).ok()? };

        let box_hdc_monitor = BoxHDC::from(&monitor_info_ex_w.szDevice);

        let scale_factor = unsafe {
            let physical_width = GetDeviceCaps(*box_hdc_monitor, DESKTOPHORZRES);
            let logical_width = GetDeviceCaps(*box_hdc_monitor, HORZRES);

            physical_width as f32 / logical_width as f32
        };

        Ok(ImplMonitor {
            hmonitor,
            monitor_info_ex_w,
            scale_factor,
        })
    }
}
