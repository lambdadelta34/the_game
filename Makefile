build:
		@cargo build --features "metal" --no-default-features

run:
		@RUST_LOG=trace cargo run

build-release:
		@cargo build --release
