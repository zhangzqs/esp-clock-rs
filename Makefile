clippy:
	cd esp32c3-impl && cargo clippy --fix --allow-dirty && cd -
	cd slint-app && cargo clippy --fix --allow-dirty && cd -
	cd libs/desktop-svc && cargo clippy --fix --allow-dirty && cd -
	cd libs/embedded-graphics-group && cargo clippy --fix --allow-dirty && cd -
	cd libs/embedded-software-slint-backend && cargo clippy --fix --allow-dirty && cd -
	cd libs/embedded-tone && cargo clippy --fix --allow-dirty && cd -
	cd server && cargo clippy --fix --allow-dirty && cd -
	cd screen-projector && cargo clippy --fix --allow-dirty && cd -
	cd desktop-simulator-impl && cargo clippy --fix --allow-dirty && cd -

cloc:
	cloc . --vcs git