SHELL:=/bin/bash

.DEFAULT_GOAL := default
.PHONY: format clippy lint shaders doc fix

BIN=game

format:
	cargo fmt

clippy:
	-cargo clippy --features="empty"

lint: format clippy
	@echo linting

metal: lint
	cargo run --features="metal" --bin="$(BIN)"

metal-release: lint
	cargo run --release --features="metal,release" --bin="$(BIN)"

metal-profiler: lint
	cargo run --release --features="metal,release,profiler" --bin="$(BIN)"

vulkan: lint
	cargo run --features="vulkan" --bin="$(BIN)"

vulkan-release: lint
	cargo run --release --features="vulkan,release" --bin="$(BIN)"

vulkan-profiler: lint
	cargo run --release --features="vulkan,release,profiler" --bin="$(BIN)"

doc:
	-cargo doc --features="empty"

fix:
	-cargo fix --allow-staged --features="empty"

default: lint

clean:
	cargo clean