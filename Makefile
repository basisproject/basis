.PHONY: all clean release build run test test-panic test-st macros

# non-versioned include
VARS ?= vars.mk
-include $(VARS)

CARGO ?= $(shell which cargo)
FEATURES ?= 
override CARGO_BUILD_ARGS += --features "$(FEATURES)"

all: build

build: 
	$(CARGO) build $(CARGO_BUILD_ARGS)

release: override CARGO_BUILD_ARGS += --release
release: build

run:
	$(CARGO) run $(CARGO_BUILD_ARGS)

test:
	$(CARGO) test $(TEST) $(CARGO_BUILD_ARGS) -- --nocapture

test-panic: override FEATURES += panic-on-error
test-panic:
	RUST_BACKTRACE=1 \
		$(CARGO) test \
			$(TEST) \
			$(CARGO_BUILD_ARGS) -- \
			--nocapture

test-st:
	$(CARGO) test $(TEST) $(CARGO_BUILD_ARGS) -- --nocapture --test-threads 1

macros:
	$(CARGO) rustc -- -Z unstable-options --pretty=expanded

clean:
	rm -rf target/
	cargo clean

