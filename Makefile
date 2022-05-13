BINARY := swhkd_wlr
BUILDFLAGS := --release
TARGET_DIR := /usr/bin
VERSION=$(shell awk -F ' = ' '$$1 ~ /version/ { gsub(/["]/, "", $$2); printf("%s",$$2) }' Cargo.toml)

all: build

build:
	@cargo build $(BUILDFLAGS)
	@cp ./target/release/$(BINARY) ./bin/$(BINARY)

install: build
	@mkdir -p $(TARGET_DIR)
	@mkdir -p /etc/$(BINARY)
	@touch /etc/$(BINARY)/$(BINARY)rc
	@cp ./bin/$(BINARY) $(TARGET_DIR)
	@chmod +x $(TARGET_DIR)/$(BINARY)

uninstall:
	@rm $(TARGET_DIR)/$(BINARY)

check:
	@cargo fmt
	@cargo check --target=x86_64-unknown-linux-musl
	@cargo clippy

release: build
	@cd bin; zip -r "$(BINARY)-x86_64-$(VERSION).zip" . ; rm ./swhkd_wlr

clean:
	@cargo clean

setup:
	@mkdir -p ./bin
	@rustup install stable
	@rustup default stable

.PHONY: check clean setup all install build release
