<p align="center">
    <img src="public/multipaste.png" width="64" />
    <br />
    <h1 align="center">Multipaste</h1>
</p>

一个朴素的 MacOS 剪切板管理工具。

大概长[这样](https://www.bilibili.com/video/BV1scDaYcEeA)。

## Features

- 朴素
- 支持图片、文字
- 记录筛选
- 记录置顶

## 配方

- UI
  - [Tauri](https://tauri.app/)
  - [UnoCSS](https://unocss.dev/)
  - [VueUse](https://vueuse.org/)
  - [PerfectScrollbar](https://www.npmjs.com/package/perfect-scrollbar)
  - [window-vibrancy](https://github.com/tauri-apps/window-vibrancy)
- 剪切板状态检测
  - [clipboard-rs](https://github.com/ChurchTao/clipboard-rs)
- 存储
  - [rusqlite](https://github.com/rusqlite/rusqlite)
  - [r2d2](https://github.com/sfackler/r2d2)
  - [diesel](https://github.com/diesel-rs/diesel)
