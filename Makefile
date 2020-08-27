SHELL:=/bin/bash

.DEFAULT_GOAL := default
.PHONY: format clippy lint server client doc fix

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

single: lint
	cargo run --release --features="metal" -- single

double: lint
	cargo run --release --features="metal" -- double

server: lint
	cargo run --release --features="metal" -- server 0.0.0.0:3456

client: lint
	cargo run --release --features="metal" -- client 127.0.0.1:3456

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