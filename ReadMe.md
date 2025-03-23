# How to Run
## Prerequisites
- A Rust toolchain (including `cargo`)
- `libclang` (for `bindgen` to work properly)
- The Linux kernel with the KVM module running
- Standard C tooling (`gcc`, `objcopy`, GNU `make`)
- An AMD64 CPU (or rewrite the assembly in `guest.S`)
## Actually Running
You should just be able to run `make`, which will
1. Assembly the assembly in `guest.S` into `guest.o`
2. Rip just the binary `.text` section from `guest.o` into `guest.bin` (VM's instruction pointer will start at the first byte of `guest.bin`)
3. Compile and run the Rust KVM hypervisor using `cargo`
