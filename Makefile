.PHONY: build

# Debug and release binaries: target/debug/lctl and target/release/lctl
build:
	cargo build
	cargo build --release
