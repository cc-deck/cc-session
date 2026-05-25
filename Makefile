.PHONY: build install test clippy clean

PREFIX ?= /usr/local

build:
	cargo build --release

install: build
	install -d $(PREFIX)/bin
	install -m 755 target/release/cc-session $(PREFIX)/bin/cc-session

test:
	cargo test

clippy:
	cargo clippy

clean:
	cargo clean
