.PHONY: all build test run lint fmt install clean

all: build

build:
	cargo build

run:
	cargo run -p work-context-manager-cli

test:
	cargo test

lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt -- --check

install:
	cargo run -p work-context-manager-installer

clean:
	cargo clean