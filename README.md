# esp-clock-rs

使用 Rust 语言编写的个人时钟，使用 Rust 开发的桌面客户端，移动 App，嵌入式软件，服务端，小工具，使用 Vue 开发 app 后台 ui 管理界面。

## 特性

- 跨平台
- 接入和风天气
- 使用 Vue+Typescript 开发 App 后台管理 ui

## 支持平台

目前已支持运行在如下平台（加粗的为重点优先适配支持的目标平台）：

### 桌面端

- - [x] **Linux**
  - [x] **Windows**
  - [x] MacOS

* 使用 embedded-graphics-simulator（基于 SDL2），基于 slint 软件渲染器逐行像素渲染到屏幕上，实现 GUI 的启动
* 使用了 reqwest 作为 http client 实现
* 使用了 rodio 作为蜂鸣器演奏框架的演奏器实现
* 直接基于 tcp 自己实现了 http server
* 使用 sled 作为 KV 存储
* 使用 env_logger 进行日志输出

### 移动端

- - [x] Android
  - [ ] iOS

* 直接使用 slint 的 Android 平台后端实现 GUI 的启动
* 使用了 reqwest 作为 http client 实现
* 使用了 rodio 作为蜂鸣器演奏框架的演奏器实现
* 直接基于 tcp 自己实现了 http server
* 使用 sled 作为 KV 存储
* 使用 android logger 进行日志输出

### MCU

- - [x] **ESP32C3**

* 使用 ST7789 驱动，通过 embedded-graphics 作为接口，基于 slint 软件渲染器逐行渲染到屏幕上，实现 GUI 的启动
* 基于 RMT 通过驱动蜂鸣器实现演奏者
* 使用了 esp-idf-svc 的 http client 作为 http client 的实现
* 使用了 esp-idf-svc 的 http server 作为 http server 的实现
* 使用 LEDC 控制屏幕亮度
* 使用 esp32-nimbe 使用蓝牙功能，实现 iOS BLE 攻击
* 使用 ESP NVS 分区作为 KV 存储
* 使用 esp logger 进行日志输出

## 服务端

- 使用 poem 框架，基于 poem-openapi 开发
- 图片服务
- 天气服务
- OpenWRT 监控

# 运行方式

## PC 桌面端模拟器

```bash
make run-on-desktop
```

## ESP32

```bash
make run-on-esp32c3
```

## Android

```bash
make run-on-android
```
