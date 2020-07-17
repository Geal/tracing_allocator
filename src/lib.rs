#![feature(allocator_api)]

extern crate libc;
extern crate time;

use std::mem;
use std::process;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::alloc::{GlobalAlloc, System, Layout, AllocErr};
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
use libc::c_void;

static mut TRACE_FD:i32 = -1;
static TRACE_ACTIVATE: AtomicBool = ATOMIC_BOOL_INIT;

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
    TRACE_ACTIVATE.store(true, Ordering::Relaxed);
  }

  pub fn deactivate() {
    TRACE_ACTIVATE.store(false, Ordering::Relaxed);
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
  if TRACE_FD != -1 && TRACE_ACTIVATE.load(Ordering::Relaxed) {
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

unsafe impl GlobalAlloc for Allocator {
  #[track_caller]
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let size = layout.size();
    let p = System.alloc(layout);
    print_size(p as usize, size, Action::Allocating);
    p
  }

  #[track_caller]
  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    print_size(ptr as usize, layout.size(), Action::Deallocating);
    System.dealloc(ptr, layout)
  }
}

