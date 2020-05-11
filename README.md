# ermalloc
This library provides a drop-in replacement for memory allocation (`malloc`, `calloc` and `realloc`) with resilience in the form of error correction and redundancy, as well as memory encryption for security. This is a software approach towards dealing with bit flips which commonly occur in radiation prone environments. Using software libraries is a low cost approach to providing resiliency in such environments, as hardware approaches such as radiation hardened devices are often very expensive for low cost missions. 

## Getting Started

Run `make` in the project directory to build the test code `test/main.c`. This creates a `build` directory from which `main` can be run.

## Features

### Memory handling API
The public API is documented in `test/ermalloc.h`:

* `er_malloc`, `er_calloc` and `er_realloc`, used to allocate memory
* `er_free`, frees memory
* `er_read_buf`, reads data into a buffer. On every read, the data is corrected using the specified resiliency policies, and if encryption in memory was specified, it is decrypted on a read. This is the only valid way to correctly access data.
* `er_write_buf`, writes data. On every write, the policies are reapplied, and if encryption is specified, then the data is stored encrypted.

### Policies

* `Redundancy`, duplicates the data as many times as specified, if duplication frequency is not specified, uses default calculated by `default_redundancy`.
* `ReedSolomon`, appends parity bits to the data of size specified, if parity bit length is not specified, uses default calculated by `default_rs`.
* `Encrypted`, encrypts data when it is stored in memory using a key known only by hardware and a nonce generated on each encryption. Uses AES-CTR-128 as this is proven to be malleable.
* Order of operation on **write**: Data is first encrypted, then parity bits are appended, and finally it is duplicated into the specified number of blocks on
* Order of operation on **read**: The block is first corrected. Reed Solomon is preferentially applied over redundancy. If Reed Solomon fails, then data is corrected by voting over the redundant bits. Finally, the data is decrypted.

### Threat Model 
This will help determine the calculation of default values of redundancy and reed solomon based on system characteristics. 

For redundancy, there is a big tradeoff between space and efficiency, and reliability. However, adding more redundancy isn't always better for reliability either, even if we were to ignore space and efficiency. This is because if the hardware isn't very reliable, after a certain point, we are introducing more errors as the space where which errors could occur increases. So an optimum value needs to be determined.

For error correction, we might want to consider different types of error correcting codes based on the program. For instance, some programs might want a code that only corrects errors in the data bits but not the parity bits, such as in a hash table (1). Another program might want the error correcting code to focus on corrections in the more significant bits. Such variations are possible because linear error codes such as Reed Solomon are highly customizable. This paper does such a thing for SEC-DAED codes, which are another class of linear block codes, however they only generate the parity matrix, not the actual code. Doing a similar thing for Reed Solomon codes and extending the paper to actually generate the syndromes from the parity matrix, thereby autogenerating the encoder and decoder for the codes would be a great follow up. 

#### Types of Errors

* `Single Event Upsets (SEU)`: This refers to a single bit flip in a semiconductor such as SRAM, DRAM, FPGA or processor triggered by a single event. SRAMs in particular are very prone to this. This is usually the case with older components where the density of the circuit is lower. For future work, we could try to determine the threshold of parameters before which these were more common than MBUs.
* `Multiple Bit Upsets (MBU)`: This refers to a burst of bits in a single word getting affected by a single event. This is increasingly becoming common as chip density increases.

#### Resiliency Characteristics
These are the factors that affect mean time to failure and patterns of failure:

* Increased chip density (more electronic components on a single chip), related to smaller charges on individual electronic components making them more sensitive to cosmic disturbances
* Memory layout, for instance memory width, makes certain error patterns more likely than others
* Age of technology, the older a processor is, the more likely it is to fail

Some programs and data are more mission critical than others. For instance, the powerboard and communications system on a satellite is significantly more mission critical than the system collecting data as the latter cannot function or be corrected with a binary upload without the former. So different types of protection models and error correcting codes must be applied to ensure a well balanced tradeoff between reliability and performance/cost.


## Testing

Preliminary testing is done via the tests in `test/main.c`. To simulate bitflips in-flight, use `flip-qemu`: https://github.com/picowar/flip-qemu. 

(1):https://scihub.wikicn.top/10.1109/TETC.2019.2953139 