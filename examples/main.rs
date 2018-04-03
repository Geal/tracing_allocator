#![feature(global_allocator)]

extern crate tracing_allocator;

use std::fs::File;

#[global_allocator]
static GLOBAL: tracing_allocator::Allocator = tracing_allocator::Allocator{};

fn main() {
  let f = File::create("trace.txt").unwrap();
  tracing_allocator::Allocator::initialize(&f);
  tracing_allocator::Allocator::activate();

  let s = String::from("Hello world!");

  let mut v = Vec::new();
  v.push(1);
}
