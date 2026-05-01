WASM_TARGET := wasm32-wasip1
WASM := target/$(WASM_TARGET)/release/zellij-autoname.wasm
PLUGIN_DIR := $(HOME)/.config/zellij/plugins

.PHONY: all build release debug install uninstall clean fmt check setup

all: release

setup:
	rustup target add $(WASM_TARGET)

build: release

release:
	cargo build --release --target $(WASM_TARGET)

debug:
	cargo build --target $(WASM_TARGET)

install: release
	mkdir -p $(PLUGIN_DIR)
	cp $(WASM) $(PLUGIN_DIR)/

uninstall:
	rm -f $(PLUGIN_DIR)/zellij-autoname.wasm

check:
	cargo check --target $(WASM_TARGET)

fmt:
	cargo fmt

clean:
	cargo clean
