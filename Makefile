SHELL := /bin/bash
.PHONY: proto

fmt:
	cargo sort -w && cargo fmt --all && cargo clippy --all-targets --all-features && cargo clippy --all-targets

fmt_check:
	cargo sort -c -w && cargo fmt --all -- --check && cargo clippy --all-targets --all-features --locked -- -D warnings && cargo clippy --all-targets --locked -- -D warnings

clean:
	cargo clean

check:
	cargo check --tests

test:
	cargo nextest run --features deadlock