# eip681-go — Go/CGo demo for witffi
#
# Prerequisites: cargo, go (1.20+)
#
# Usage:
#   make        — build Rust + Go, then run the demo
#   make build  — build only (Rust staticlib + Go binary)
#   make test   — build Rust and run Go tests
#   make run    — run the demo (assumes already built)
#   make clean  — remove build artifacts

REPO_ROOT := ../..
LIB_DIR   := $(REPO_ROOT)/target/debug

CGO_ENV := CGO_LDFLAGS="-L$(abspath $(LIB_DIR))"

.PHONY: all build build-rust build-go test run clean

all: build run

build: build-rust build-go

build-rust:
	cargo build -p eip681-ffi

build-go: build-rust
	$(CGO_ENV) go build -o eip681-example ./cmd/eip681-example

test: build-rust
	$(CGO_ENV) go test -v

run:
	$(CGO_ENV) go run ./cmd/eip681-example

clean:
	rm -f eip681-example
