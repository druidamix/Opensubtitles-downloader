prog :=osd


default: build
	
release :=--release
target :=release

build:
	cargo build $(release)

install:
	cp target/$(target)/$(prog) ~/.local/bin/$(prog)
	strip ~/.local/bin/$(prog)

uninstall:
	rm ~/.local/bin/osd

clean:
	rm -rf target

all: build install

help:
	@echo "usage: make && make install"
