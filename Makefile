GCC_BIN ?= $(shell which gcc)
CARGO_BIN ?= $(shell which cargo)

build: build-c

.PHONY: build-rust
build-rust:
	$(CARGO_BIN) +nightly build

build-ermalloc_c: src/ermalloc.c src/ermalloc.h
	mkdir -p build
	$(GCC_BIN) -Og -g -c -fpic -o build/ermalloc_c.o src/ermalloc.c -I src/
	$(GCC_BIN) -shared -o build/libermalloc_c.so build/ermalloc_c.o

build-c: src/main.c build-ermalloc_c build-rust
	$(GCC_BIN) -Og -g -Ltarget/debug/ -o build/main src/main.c build/ermalloc_c.o -l:libermalloc.a -pthread -ldl

.PHONY: clean
clean:
	rm -rf build/
