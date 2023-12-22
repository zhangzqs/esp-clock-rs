# esp-clock-rs
使用Rust语言编写的个人时钟，使用Rust开发的桌面客户端，移动App，嵌入式软件，服务端，小工具，使用Vue开发app后台ui管理界面。

目前已支持运行在如下平台（加粗的为重点优先适配支持的目标平台）：
1. 桌面端
    - [x] **Linux**
    - [x] Windows
    - [x] MacOS

2. 移动端
    - [x] Android
    - [ ] iOS
3. MCU
    - [x] **ESP32C3**

# 运行方式

## PC桌面端模拟器
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
