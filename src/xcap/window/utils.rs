use sysinfo::System;

use super::error::XCapResult;

pub(super) fn wide_string_to_string(wide_string: &[u16]) -> XCapResult<String> {
    let string = if let Some(null_pos) = wide_string.iter().position(|pos| *pos == 0) {
        String::from_utf16(&wide_string[..null_pos])?
    } else {
        String::from_utf16(wide_string)?
    };

    Ok(string)
}

pub(super) fn get_os_major_version() -> u8 {
    System::os_version()
        .map(|os_version| {
            let strs: Vec<&str> = os_version.split(' ').collect();
            strs[0].parse::<u8>().unwrap_or(0)
        })
        .unwrap_or(0)
}
