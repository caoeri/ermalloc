GCC_BIN ?= $(shell which gcc)
CARGO_BIN ?= $(shell which cargo)

build: build-c

.PHONY: build-rust
build-rust:
	$(CARGO_BIN) +nightly build

build-ermalloc_c: test/ermalloc.c test/ermalloc.h
	mkdir -p build
	$(GCC_BIN) -Og -g -c -fpic -o build/ermalloc_c.o test/ermalloc.c -I test/
	$(GCC_BIN) -shared -o build/libermalloc_c.so build/ermalloc_c.o

build-c: test/main.c build-ermalloc_c build-rust
	$(GCC_BIN) -Og -g -Ltarget/debug/ -o build/main test/main.c build/ermalloc_c.o -l:libermalloc.a -pthread -ldl

.PHONY: clean
clean:
	rm -rf build/
