#![feature(global_allocator, allocator_api, heap_api)]

extern crate libc;


use std::heap::{Alloc, System, Layout, AllocErr};
use std::fs::File;
use std::mem;
use std::os::unix::io::AsRawFd;
use libc::c_void;
use std::process;


static mut TRACE_FD:i32 = 0;

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

unsafe impl<'a> Alloc for &'a Allocator {
  unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
    let size = layout.size();
    let res = System.alloc(layout);
    if let Ok(p) = res {
      if TRACE_FD != 0 {
        let mut buf: [u8; 35] = unsafe { mem::uninitialized() };

        let psz = mem::size_of::<usize>();
        let mut counter = 8;
        let mut index = 0;

        let shr = (psz as u32)*8 - 8;

        let c: usize = p as usize;
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

        buf[index] = b':';
        buf[index+1] = b' ';
        index += 2;

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

        unsafe {
          libc::write(TRACE_FD,
            buf.as_ptr() as *const c_void,
            buf.len())
        };
      }
    }
    res
  }

  unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
    System.dealloc(ptr, layout)
  }
}

