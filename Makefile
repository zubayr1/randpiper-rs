.PHONY: all debug release

debug:
	cargo build --all

release:
	cargo build --release --all