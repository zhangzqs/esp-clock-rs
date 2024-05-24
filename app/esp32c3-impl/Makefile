save-image:
	cargo build -r
	espflash save-image --chip esp32c3 target/riscv32imc-esp-espidf/release/esp32c3-impl output-release.bin

save-debug-image:
	cargo build
	espflash save-image --chip esp32c3 target/riscv32imc-esp-espidf/debug/esp32c3-impl output-debug.bin