<p align="center">
    <img src="public/multipaste.png" width="64" />
    <br />
    <h1 align="center">Multipaste</h1>
</p>

## 简介

[一个朴素的 MacOS 剪切板管理工具](https://tkzt.cn/blogs/240913_multipaste)。

大概长[这样](https://www.bilibili.com/video/BV1scDaYcEeA)。

## 风味

- 朴素
- 支持图片、文字
- 记录筛选
- 记录置顶

## 配料表

- UI
  - [Tauri](https://tauri.app/)
  - [Vue](https://vuejs.org/)
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
 
## 过敏原信息

- 基于跨端框架的 MacOS 应用
- 存在 Raw SQL
