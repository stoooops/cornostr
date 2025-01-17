.PHONY: all build up down clean lint fmt fmtcheck run

# Default target
all: lint fmt

lint:
	cargo clippy --no-deps --all-targets --all-features --future-incompat-report -- -D warnings

fmt:
	cargo fmt

fmtcheck:
	cargo fmt -- --check

clean:
	cargo clean

run:
	cargo run -- wss://relay.damus.io

# Add these targets if they're relevant to your project
build:
	cargo build

up:
	@echo "Define your 'up' command here"

down:
	@echo "Define your 'down' command here"

relay:
	cargo run -- relay --address 127.0.0.1:8080

subscribe:
	cargo run -- client --relay ws://127.0.0.1:8080 subscribe

publish:
	cargo run -- client --relay ws://127.0.0.1:8080 publish --message "Hello, Nostr!"
