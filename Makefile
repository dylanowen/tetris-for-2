SHELL:=/bin/bash

.DEFAULT_GOAL := default
.PHONY: format clippy lint doc fix

format:
	cargo fmt

clippy:
	-cargo clippy --features="empty"

lint: format clippy
	@echo linting

build: lint
	cargo build --features="metal"

release: lint
	cargo build --release --features="metal,release"

profiler: lint
	cargo build --release --features="metal,release,profiler"

#vulkan: lint
#	cargo run --features="vulkan"
#
#vulkan-release: lint
#	cargo run --release --features="vulkan,release"
#
#vulkan-profiler: lint
#	cargo run --release --features="vulkan,release,profiler"

doc:
	-cargo doc --features="empty"

fix:
	-cargo fix --allow-staged --features="empty"

default: lint

clean:
	cargo clean