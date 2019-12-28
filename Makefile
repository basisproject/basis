.PHONY: all clean release build run test test-panic test-st macros

# non-versioned include
VARS ?= vars.mk
-include $(VARS)

CARGO ?= $(shell which cargo)
FEATURES ?= with-serde
TARGET := ./target/debug/basis
BASIS_DB ?= /tmp/basis-db
override CARGO_BUILD_ARGS += --features "$(FEATURES)"

all: build

build:
	$(CARGO) build $(CARGO_BUILD_ARGS)

release: override CARGO_BUILD_ARGS += --release
release: build

run:
	$(CARGO) run $(CARGO_BUILD_ARGS) -- run -d $(BASIS_DB) -c config/block/0/node.toml --consensus-key-pass pass --service-key-pass pass

run-release: override CARGO_BUILD_ARGS += --release
run-release:
	$(CARGO) run $(CARGO_BUILD_ARGS) -- run -d $(BASIS_DB) -c config/block/0/node.toml --consensus-key-pass pass --service-key-pass pass

clean-db:
	rm -rf $(BASIS_DB)

run-clean: clean-db run

reconfig: all
	mkdir -p config/block/
	$(CARGO) run $(CARGO_BUILD_ARGS) -- generate-template config/block/template.toml --validators-count=1
	$(CARGO) run $(CARGO_BUILD_ARGS) -- generate-config config/block/template.toml config/block/0/ --peer-address 127.0.0.1:6969 --consensus-key-pass pass --service-key-pass pass
	$(CARGO) run $(CARGO_BUILD_ARGS) -- finalize config/block/0/sec.toml config/block/0/node.toml --public-api-address 0.0.0.0:13007 --private-api-address 0.0.0.0:13008 --public-configs config/block/0/pub.toml

test-release: override CARGO_BUILD_ARGS += --release
test-release:
	$(CARGO) test $(TEST) $(CARGO_BUILD_ARGS) -- --nocapture

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
	$(CARGO) rustc $(CARGO_BUILD_ARGS) -- --pretty=expanded

clean:
	rm -rf target/
	cargo clean

