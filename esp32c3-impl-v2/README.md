# 硬件方案

硬件方案使用现成的 pyClock

具体 PCB 布局在以下仓库:
https://github.com/01studio-lab/pyClock

# 引脚布局

esp32c3 模组引出了以下 GPIO 外设：

## 已被占用的 GPIO:

| GPIO 编号 | 已连接的外设        |
| --------- | ------------------- |
| GPIO0     | Sensor 外部接口引出 |
| GPIO2     | 内部蓝色 LED        |
| GPIO4     | LCD D/C             |
| GPIO5     | LCD CS              |
| GPIO6     | LCD SCL             |
| GPIO7     | LCD SDA             |
| GPIO8     | LCD RST             |
| GPIO9     | 外部按键            |
| GPIO18    | USB-                |
| GPIO19    | USB+                |
| GPIO20    | RX0                 |
| GPIO21    | TX0                 |

## 剩余可用的 GPIO

根据官方手册，以下未使用的 GPIO 支持如下功能：

| GPIO 编号 | 支持                        |
| --------- | --------------------------- |
| GPIO1     | GPIO1, ADC1_CH1, XTAL_32K_N |
| GPIO3     | GPIO3, ADC1_CH3, LED PWM    |
| GPIO10    | GPIO10, FSPICS0, LED PWM    |

# 魔改

| GPIO 编号 | 使用                              |
| --------- | --------------------------------- |
| GPIO0     | 连接蜂鸣器                        |
| GPIO10    | 连接屏幕背光，可 PWM 调节屏幕亮度 |
