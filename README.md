# Tracing allocator for Rust

This project allows you to log all the allocations to a file.

To include it in your project, initialize the library with a file like this:

```rust
#![feature(global_allocator)]

extern crate tracing_allocator;

use std::fs::File;

#[global_allocator]
static GLOBAL: tracing_allocator::Allocator = tracing_allocator::Allocator{};

fn main() {
  let f = File::create("trace.txt").unwrap();
  tracing_allocator::Allocator::initialize(&f);

  let s = String::from("Hello world!");

  let mut v = Vec::new();
  v.push(1);
```

The `trace.txt` file will then have the following content:

```rust
00029801ACDA259B A 00007FB780500000 000000000000000C
00029801ACDB7EFB A 00007FB780500010 0000000000000010
00029801ACDBAAC1 D 00007FB780500010 0000000000000010
00029801ACDBCD09 D 00007FB780500000 000000000000000C
```

Columns:

- time (monotonic, so not linked to any timezone)
- `A` for allocation, `D` for deallocation
- memory address
- size

