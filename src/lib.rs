#![feature(global_allocator, allocator_api, heap_api)]

extern crate libc;
extern crate time;

use std::heap::{Alloc, System, Layout, AllocErr};
use std::fs::File;
use std::mem;
use std::os::unix::io::AsRawFd;
use libc::c_void;
use std::process;

static mut TRACE_FD:i32 = -1;
static mut TRACE_ACTIVATE:bool = false;

pub struct Allocator {
}

impl Allocator {
  pub fn new() -> Allocator {
    Allocator {
    }
  }

  pub fn initialize(f: &File) {
    unsafe {
      TRACE_FD = f.as_raw_fd();
    }
  }

  pub fn activate() {
    unsafe {
      TRACE_ACTIVATE = true;
    }
  }

  pub fn deactivate() {
    unsafe {
      TRACE_ACTIVATE = false;
    }
  }
}

fn to_hex(i: u8) -> u8 {
  if i > 15 {
    process::exit(42);
  }

  if i < 10 {
    48 + i
  } else {
    (i - 10) + 65
  }
}

enum Action {
  Allocating,
  Deallocating
}

unsafe fn print_size(address: usize, size: usize, action: Action) {
  if TRACE_FD != -1 && TRACE_ACTIVATE {
    let mut buf: [u8; 100] = mem::uninitialized();
    let psz = mem::size_of::<usize>();
    let shr = (psz as u32)*8 - 8;
    let mut index = 0;

    let mut counter = 8;

    let c = time::precise_time_ns() as usize;
    loop {
      if counter == 0 {
        break;
      }

      let current = ((c.overflowing_shl(64 - 8*counter as u32).0).overflowing_shr(shr).0) as u8;
      buf[index] = to_hex(current >> 4);
      buf[index+1] = to_hex( (current << 4) >> 4);

      counter -= 1;
      index += 2;
    }

    buf[index] = b' ';
    index += 1;

    match action {
      Action::Allocating => buf[index] = b'A',
      Action::Deallocating => buf[index] = b'D',
    };

    buf[index+1] = b' ';
    index += 2;

    counter = 8;
    let c = address;
    loop {
      if counter == 0 {
        break;
      }

      let current = ((c.overflowing_shl(64 - 8*counter as u32).0).overflowing_shr(shr).0) as u8;
      buf[index] = to_hex(current >> 4);
      buf[index+1] = to_hex( (current << 4) >> 4);

      counter -= 1;
      index += 2;
    }

    buf[index] = b' ';
    index += 1;

    counter = 8;
    loop {
      if counter == 0 {
        break;
      }

      let current = ((size.overflowing_shl(64 - 8*counter as u32).0).overflowing_shr(shr).0) as u8;
      buf[index] = to_hex(current >> 4);
      buf[index+1] = to_hex( (current << 4) >> 4);

      counter -= 1;
      index += 2;
    }
    buf[index] = b'\n';

    libc::write(TRACE_FD,
      buf.as_ptr() as *const c_void,
      index+1);
      //buf.len());
  }

}

unsafe impl<'a> Alloc for &'a Allocator {
  unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
    let size = layout.size();
    let res = System.alloc(layout);
    if let Ok(p) = res {
      print_size(p as usize, size, Action::Allocating);
    }
    res
  }

  unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
    print_size(ptr as usize, layout.size(), Action::Deallocating);
    System.dealloc(ptr, layout)
  }
}

