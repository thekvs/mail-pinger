INSTALLDIR:=$(shell mktemp --directory --tmpdir mail-pinger.XXXXXXXXXX)
TMPDIR:=$(shell mktemp --directory --tmpdir mail-pinger.XXXXXXXXXX)

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
.DEFAULT_GOAL := build

clean:
	rm -f *.deb
	rm -rf target

check_env:
	if ! which docker; then \
		echo ERROR: docker is not installed!; \
		exit 1; \
	fi
	curl -L https://github.com/a8m/envsubst/releases/download/v1.2.0/envsubst-`uname -s`-`uname -m` -o $(TMPDIR)/envsubst
	chmod +x $(TMPDIR)/envsubst

build: check_env
	cargo build --release

deb: clean build
	VERSION=$(VERSION) PATCH=$(PATCH) $(TMPDIR)/envsubst -i nfpm.yaml.in -o nfpm.yaml
	docker run --user `id -u`:`id -g` \
		--rm \
        --volume $(PWD):/tmp/pkg \
        --workdir /tmp/pkg \
        goreleaser/nfpm pkg --packager deb --target .
	rm -rf $(TMPDIR)
	rm -f nfpm.yaml
