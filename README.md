# ermalloc
This library provides a drop-in replacement for memory allocation (`malloc`, `calloc` and `realloc`) with resilience in the form of error correction and redundancy, as well as memory encryption for security.

## Getting Started

Run `make` in the project directory to build the test code `test/main.c`. This creates a `build` directory from which `main` can be run.

## Features

#### Memory handling API
The public API is documented in `test/ermalloc.h`:

* `er_malloc`, `er_calloc` and `er_realloc`, used to allocate memory
* `er_free`, frees memory
* `er_read_buf`, reads data into a buffer. On every read, the data is corrected using the specified resiliency policies, and if encryption in memory was specified, it is decrypted on a read. This is the only valid way to correctly access data.
* `er_write_buf`, writes data. On every write, the policies are reapplied, and if encryption is specified, then the data is stored encrypted.

#### Policies

* `Redundancy`, duplicates the data as many times as specified
* `ReedSolomon`, appends parity bits to the data of size specified
* `Encrypted`, encrypts data when it is stored in memory using a key known only by hardware and a nonce generated on each encryption. Uses AES-CTR-128 as this is proven to be malleable.
* Order of operation on **write**: Data is first encrypted, then parity bits are appended, and finally it is duplicated into the specified number of blocks on
* Order of operation on **read**: The block is first corrected. Reed Solomon is preferentially applied over redundancy. If Reed Solomon fails, then data is corrected by voting over the redundant bits. Finally, the data is decrypted.

## Testing

Preliminary testing is done via the tests in `test/main.c`. To simulate bitflips in-flight, use `flip-qemu`: https://github.com/picowar/flip-qemu. 
