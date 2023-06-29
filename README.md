# esp-clock-rs
使用Rust语言编写的个人时钟，计划支持PC端模拟器，可移植性
目前暂时仅处于探索阶段

# 运行方式

## PC端模拟器
```bash
cd desktop-simulator-impl
cargo run
```

## ESP32
```bash
cd esp32-impl
cargo build 
espflash target/riscv32imc-esp-espidf/debug/esp32c3-impl --monitor
```