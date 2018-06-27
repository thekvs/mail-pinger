VERSION="0.1.0"
INSTALLDIR:=$(shell mktemp --directory --tmpdir mail-pinger.XXXXXXXXXX)

.PHONY: clean deb build check_env
.DEFAULT_GOAL := deb

clean:
	cargo clean
	rm -f *.deb

check_env:
	if ! which fpm; then \
		echo ERROR: fpm is not installed!; \
		exit 1; \
	fi

build: check_env
	cargo build --release

deb: build
	mkdir -p $(INSTALLDIR)/usr/local/bin/
	cp -a ./target/release/mail-pinger $(INSTALLDIR)/usr/local/bin/
	fpm -s dir -t deb -n "mail-pinger" \
		--version $(VERSION) \
		--description "Mail pinger" \
		--deb-compression xz \
		--maintainer "Konstantin Sorokin <kvs@sigterm.ru>" \
		-C $(INSTALLDIR)
	rm -rf $(INSTALLDIR)
