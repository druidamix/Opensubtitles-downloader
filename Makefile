prog :=osd


  release :=--release
  target :=release

build:
	cargo build $(release)

install:
	cp target/$(target)/$(prog) ~/.local/bin/$(prog)

clean:
	rm -rf target
	
all: build install
 
help:
	@echo "usage: make install"
