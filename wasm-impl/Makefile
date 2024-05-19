all: release

serve:
	python3 -m http.server 1808

dev:
	cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --target web --dev"

release:
	wasm-pack build --target web --release