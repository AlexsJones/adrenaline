.PHONY: server client

server:
	cargo run --example server
client:
	cargo run --example client
file:
	cargo run --example file