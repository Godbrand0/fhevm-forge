.PHONY: build publish

build:
	cargo build --release

publish:
	cargo publish

install:
	cargo install --path .
