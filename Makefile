all: build

build:
	cargo build --release

install:
	mkdir -p /usr/bin/
	mkdir -p /etc/axact/
	mkdir -p /lib/systemd/system/
	cp -rT src/assets /etc/axact/static
	cp target/release/axact /usr/bin
	cp debian/service /lib/systemd/system/axact.service
	systemctl start axact.service && systemctl enable axact.service

clean:
	cargo clean
