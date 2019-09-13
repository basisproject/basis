.PHONY: all build clean crun test test-st run

rustfiles := $(shell find src -name "*.rs")

# non-versioned include
-include ../../vars.mk

CARGO := $(shell which cargo)
CARGO_BUILD_ARGS :=

all: build

build: 
	cargo build

run: build
	cargo run

test:
	cargo test $(TEST) $(CARGO_BUILD_ARGS) -- --nocapture

test-st:
	RUST_TEST_TASKS=1 cargo test $(TEST) $(CARGO_BUILD_ARGS) -- --nocapture

