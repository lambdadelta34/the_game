build:
		@cargo build

run:
		@RUST_LOG=trace cargo run

build-release:
		@cargo build --release
