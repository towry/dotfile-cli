
all: debug

debug:
	cargo build --features debug

release:
	cargo build --release

.PHONY: debug release
