# esp-clock-rs
使用Rust语言编写的个人时钟，使用Rust开发的桌面客户端，移动App，嵌入式软件，服务端，小工具，使用Vue开发app后台ui管理界面。

## 支持平台
目前已支持运行在如下平台（加粗的为重点优先适配支持的目标平台）：
### 桌面端
    - [x] **Linux**
    - [x] Windows
    - [x] MacOS
- 使用embedded-graphics-simulator（基于SDL2），基于slint软件渲染器逐行渲染到屏幕上，实现GUI的启动
- 使用了reqwest作为http client实现
- 使用了rodio作为蜂鸣器演奏框架的演奏者实现
- 直接使用tcp自己实现了http client
  
### 移动端
    - [x] Android
    - [ ] iOS
- 直接使用slint的Android平台后端实现GUI的启动
- 使用了rodio作为蜂鸣器演奏框架的演奏者实现
- 直接使用tcp自己实现了http client/server

### MCU
    - [x] **ESP32C3**
- 使用ST7789驱动，通过embedded-graphics作为接口，基于slint软件渲染器逐行渲染到屏幕上，实现GUI的启动
- 基于RMT通过驱动蜂鸣器实现演奏者
- 使用了esp-idf-svc的http client 作为http client的实现
- 使用了esp-idf-svc的http server 作为http server的实现
- 使用LEDC控制屏幕亮度
- 使用esp32-nimbe使用蓝牙功能，实现iOS BLE攻击

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
