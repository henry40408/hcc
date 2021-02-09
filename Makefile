.PHONY: build-docker-image

build-docker-image:
	cross build --release --target x86_64-unknown-linux-musl
	strip target/x86_64-unknown-linux-musl/release/potential-giggle-server
	docker build -t henry40408/potential-giggle .
