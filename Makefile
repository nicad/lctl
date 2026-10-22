.PHONY: build test test-all

# The Command Line Tools can ship an SDK newer than their linker understands (linking then
# fails with "unknown architecture" in libSystem.tbd). Build against the SDK matching the
# running macOS when there is one, unless SDKROOT is already set.
MACOS_MAJOR := $(shell sw_vers -productVersion | cut -d. -f1)
SDK_FOR_THIS_MACOS := $(dir $(shell xcrun --show-sdk-path))MacOSX$(MACOS_MAJOR).sdk
ifeq ($(origin SDKROOT),undefined)
  ifneq ($(wildcard $(SDK_FOR_THIS_MACOS)),)
    export SDKROOT := $(SDK_FOR_THIS_MACOS)
  endif
endif

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
