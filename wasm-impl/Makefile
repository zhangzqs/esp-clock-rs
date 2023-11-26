build: 
	wasm-pack build --debug --target web

release:
	wasm-pack build --release --target web

run: build
	python3 -m http.server