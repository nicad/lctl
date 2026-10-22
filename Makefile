.PHONY: build test test-all

# Debug and release binaries: target/debug/lctl and target/release/lctl
build:
	cargo build
	cargo build --release

# Unit tests + GUI e2e; no sudo (system-domain tests are #[ignore]d)
test:
	cargo test

# Everything, including system-domain e2e: asks for sudo once up front, and the
# interactive tests prompt again mid-run. Run from a real terminal; never `sudo make`.
test-all:
	sudo -v
	cargo test -- --include-ignored --test-threads=1
