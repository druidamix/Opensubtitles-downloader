prog :=osd


release :=--release
target :=release

build:
	cargo build $(release)

install:
	cp target/$(target)/$(prog) ~/.local/bin/$(prog)

uninstall:
	rm ~/.local/bin/osd

clean:
	rm -rf target

all:  
	ifeq (, $(shell command -v tldr 2> /dev/null))
		$(error "No cargo command found. Please install cargo.")
	endif
	build install

help:
	@echo "usage: make install"
