make-output-dir:
	mkdir -p target/output

build-debug-hex: make-output-dir 
	cargo build
	espflash save-image --chip esp32c3 target/riscv32imc-esp-espidf/debug/esp32c3-impl target/output/esp32c3-impl-debug.hex

build-release-hex: make-output-dir
	cargo build --release
	espflash save-image --chip esp32c3 target/riscv32imc-esp-espidf/release/esp32c3-impl target/output/esp32c3-impl-release.hex