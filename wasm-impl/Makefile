build: pkg/wasm_impl_bg.wasm pkg/wasm_impl.js
	wasm-pack build --release --target web

run: build
	cd pkg && python3 -m http.server