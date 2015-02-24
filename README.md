# ld9
A crude cross-linker from OS X to Plan 9.

It reads a Mach-O into memory, checks that it isn't dynamically linked, and blindly copies bytes into a Plan 9 executable.

## Build

Run the commands:
```sh
cargo build --release
```

The executable will be placed at `target/release/ld9`

## Example

Using the following file, named `crt0.s`:
```asm
.globl start

start:
    mov     $8, %eax
    int     $0x40
    jmp start
```

Run the commands:
```sh
clang -static -fPIC -c crt.s
ld -static -o main crt.o
ld9
```

This will produce a Plan 9 executable called `aout9` that, when run on Plan 9,
immediately calls `exits`.
