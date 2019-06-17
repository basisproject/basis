.PHONY: all clean release build run test test-panic test-st macros

# non-versioned include
VARS ?= vars.mk
-include $(VARS)

CARGO ?= $(shell which cargo)
FEATURES ?= with-serde
TARGET := ./target/debug/conductor
override CARGO_BUILD_ARGS += --features "$(FEATURES)"

all: build

build:
	$(CARGO) build $(CARGO_BUILD_ARGS)

release: override CARGO_BUILD_ARGS += --release
release: build

run:
	$(CARGO) run $(CARGO_BUILD_ARGS) -- run --node-config config/block/config.toml --db-path play/db/ --public-api-address 0.0.0.0:13007 --consensus-key-pass pass --service-key-pass pass

reconfig: all
	mkdir -p config/block/
	$(CARGO) run $(CARGO_BUILD_ARGS) -- generate-template config/block/common.toml --validators-count=1
	$(CARGO) run $(CARGO_BUILD_ARGS) -- generate-config config/block/common.toml config/block/node.pub.toml config/block/node.sec.toml --peer-address 127.0.0.1:6969 -c config/block/consensus.toml -s config/block/service.toml -n
	$(CARGO) run $(CARGO_BUILD_ARGS) -- finalize --public-api-address 0.0.0.0:13007 --private-api-address 0.0.0.0:13008 config/block/node.sec.toml config/block/config.toml --public-configs config/block/node.pub.toml

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

