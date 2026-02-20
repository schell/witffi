# eip681-kotlin — Kotlin/JNI demo for witffi
#
# Prerequisites: cargo, kotlinc, java (all available via Homebrew)
#
# Usage:
#   make        — build Rust + Kotlin, then run the demo
#   make build  — build only (Rust cdylib + Kotlin jar)
#   make run    — run the demo (assumes already built)
#   make clean  — remove build artifacts

REPO_ROOT := ../..
LIB_DIR   := $(REPO_ROOT)/target/debug

.PHONY: all build build-rust build-kotlin run clean

all: build run

build: build-rust build-kotlin

build-rust:
	cargo build -p eip681-ffi

build-kotlin: build-rust
	kotlinc src/*.kt -include-runtime -d demo.jar 2>/dev/null

run:
	java \
		-Djava.library.path=$(LIB_DIR) \
		--enable-native-access=ALL-UNNAMED \
		-jar demo.jar

clean:
	rm -f demo.jar
