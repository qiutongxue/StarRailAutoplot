# 「崩坏：星穹铁道」自动对话程序

> 灵感来自[三月七小助手](https://github.com/moesnow/March7thAssistant)的自动对话功能。
> 由于本人只使用三月七的自动对话功能，每次启用自动对话都要经历长时间的主程序加载，所以想把自动对话功能独立出来。
> 但是拆分功能之后发现 Python 打包体积过于庞大（需要30MB），遂尝试使用 Rust 重写。

- 📺 支持全屏/窗口化以及多种 16:9 分辨率
- ⚡ 不到 500KB，极速启动
- ⌨️ 大大增加鼠标和空格键寿命
- 🤲 解放双手，上个厕所也不耽误事儿
  
## 使用须知

**请务必使用管理员身份运行！** **请务必使用管理员身份运行！** **请务必使用管理员身份运行！**

> 截屏和鼠标模拟都仅在管理员模式下才能正常工作
 
运行程序之后，保持游戏窗口在前台即可，进入对话后会自动点击

**游戏窗口化运行时请确保鼠标在窗口内**

## 下载

请前往 [Releases](https://github.com/qiutongxue/sr_plot_rs/releases) 页面下载最新的可执行文件版本。 

或下载源码后使用 cargo 编译运行。（需要按照 [OpenCV-rust](https://crates.io/crates/opencv) 的说明下载 OpenCV 和 LLVM 并设置相应的环境变量）

## 致谢

- 感谢 [三月七小助手](https://github.com/moesnow/March7thAssistant) 提供的灵感
- 感谢 [xcap](https://github.com/nashaofu/xcap) 的截图功能，项目中的 xcap 模块魔改于此
