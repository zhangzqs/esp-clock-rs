all: debug

serve:
	python3 -m http.server 1808

watch:
	cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --target web --dev"

debug:
	wasm-pack build --target web --dev

release:
	wasm-pack build --target web --release