.PHONY: all build test run lint fmt docs install clean

CARGO ?= $(shell command -v cargo 2>/dev/null || echo "$(HOME)/.cargo/bin/cargo")

all: build

build:
	$(CARGO) build

run:
	$(CARGO) run -p context-manager-cli

test:
	$(CARGO) test

docs:
	$(CARGO) doc --no-deps

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

fmt:
	$(CARGO) fmt -- --check

install:
	$(CARGO) run -p context-manager-installer

clean:
	$(CARGO) clean