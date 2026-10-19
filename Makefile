.PHONY: all clean

all:
	cargo build --release --locked --manifest-path projects/proj1/src/Cargo.toml

clean:
	cargo clean --manifest-path projects/proj1/src/Cargo.toml
