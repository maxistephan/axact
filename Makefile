# Define the default target
all: build

# Define the build target
build:
	cargo build --release

# Define the install target
install:
	cargo install --path .
	mkdir -p /etc/axact/
	cp -rT src/assets /etc/axact/static

# Define the clean target
clean:
	cargo clean