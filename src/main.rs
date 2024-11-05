use simple_logger::SimpleLogger;
use sr_plot_rs::plot::Plot;
use std::{fmt::Write, thread};
use unicode_width::UnicodeWidthStr;

const WELCOME: &str = r#"
欢迎使用「崩坏：星穹铁道」自动对话程序

请使用「管理员身份」运行此程序
需要保持游戏窗口在前台运行
若游戏为「窗口化」，请确保鼠标位置在游戏窗口内
"#;

fn main() {
    setup();
    println!("{}", hr(WELCOME));
    thread::spawn(|| Plot::default().run()).join().unwrap();
}

#[cfg(not(debug_assertions))]
fn setup() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_local_timestamps()
        .with_timestamp_format(time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
        ))
        .init()
        .unwrap();
}

#[cfg(debug_assertions)]
fn setup() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .with_local_timestamps()
        .with_timestamp_format(time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
        ))
        .env()
        .init()
        .unwrap();
}

fn hr(title: &str) -> String {
    let mut output = String::new();
    // 左右留下长度为 10 的空格
    let max_length = title.split("\n").map(|line| line.width()).max().unwrap() + 22;

    writeln!(&mut output, "+{}+", "-".repeat(max_length - 2)).unwrap();
    for line in title.split("\n") {
        let length = line.width();
        let left_padding = (max_length - length - 2) / 2;
        let right_padding = max_length - length - left_padding - 2;
        writeln!(
            &mut output,
            "|{}{}{}|",
            " ".repeat(left_padding),
            line,
            " ".repeat(right_padding)
        )
        .unwrap();
    }
    writeln!(&mut output, "+{}+", "-".repeat(max_length - 2)).unwrap();

    output
}
