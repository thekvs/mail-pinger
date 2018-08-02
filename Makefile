INSTALLDIR:=$(shell mktemp --directory --tmpdir mail-pinger.XXXXXXXXXX)

VERSION := $(shell git describe | cut -d- -f1)
RECORDS := $(shell git describe | grep -o "-" | wc -l)
ifeq ($(RECORDS), 2)
        PATCH := $(shell git describe | cut -d- -f2)
        LAST_GIT_COMMIT := $(shell git describe | cut -d- -f3)
else
        # If we tagged the very last commit then output of the 'git describe' command will not have
        # number of additional commits on top of the tagged object and hash of the latest commit parts.
        # Thus we need to handle such situation by explicitly specifing PATCH and LAST_GIT_COMMIT.
        PATCH := 0
        LAST_GIT_COMMIT := g$(shell git rev-parse --short HEAD)
endif

.PHONY: clean deb build check_env
.DEFAULT_GOAL := deb

clean:
	rm -f *.deb

check_env:
	if ! which fpm; then \
		echo ERROR: fpm is not installed!; \
		exit 1; \
	fi

build: check_env
	cargo build --release

deb: clean build
	mkdir -p $(INSTALLDIR)/usr/local/bin/
	cp -a ./target/release/mail-pinger $(INSTALLDIR)/usr/local/bin/
	fpm --input-type dir \
		--output-type deb \
		--name "mail-pinger" \
		--version $(VERSION).$(PATCH)+$(LAST_GIT_COMMIT) \
		--description "Mail pinger" \
		--deb-compression xz \
		--maintainer "Konstantin Sorokin <kvs@sigterm.ru>" \
		--chdir $(INSTALLDIR)
	rm -rf $(INSTALLDIR)
