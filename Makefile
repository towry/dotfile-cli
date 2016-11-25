
all: debug

debug:
	cargo build --features debug

release:
	cargo build --release

install:
	install ./target/release/dotfile /usr/local/bin/

.PHONY: debug release install
