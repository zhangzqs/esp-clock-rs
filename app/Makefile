fmt:
	cd proto && cargo fmt && cd -
	cd app-core && cargo fmt && cd -
	cd desktop-impl && cargo fmt && cd -
	cd desktop-software-renderer-impl && cargo fmt && cd -
	cd esp32c3-impl && cargo fmt && cd -
	cd wasm-impl && cargo fmt && cd -

clippy:
	cd proto && cargo clippy --fix --allow-dirty && cd -
	cd app-core && cargo clippy --fix --allow-dirty && cd -
	cd desktop-impl && cargo clippy --fix --allow-dirty && cd -
	cd desktop-software-renderer-impl && cargo clippy --fix --allow-dirty && cd -
	cd esp32c3-impl && cargo clippy --fix --allow-dirty && cd -
	cd wasm-impl && cargo clippy --fix --allow-dirty && cd -