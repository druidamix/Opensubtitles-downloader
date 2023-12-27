prog :=osd


default: build
	
release :=--release
target :=release
DESTDIR := ~/.local/bin

build:
	cargo build $(release)

install-relsease:
	cp target/$(target)$(prog) $(DESTDIR)/$(prog)
	strip /usr/bin/$(prog)

install:
	cp target/$(target)/$(prog) $(DESTDIR)/$(prog)
	strip $(DESTDIR)/$(prog)

uninstall:
	rm $(DESTDIR)/osd

clean:
	rm -rf target

all: build install

help:
	@echo "usage: make && make install"
