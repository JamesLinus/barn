#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(const_fn)]
#![no_std]
#![feature(alloc)]
#![feature(rand)]
#![feature(associated_type_defaults)]

#[cfg(any(test, feature="hosted"))]
#[macro_use]
extern crate std;

macro_rules! debug {
    ($fmt:expr) => {
      println!(concat!("DEBUG: ", $fmt));
    };
    ($fmt:expr, $($arg:tt)*) => {
      println!(concat!("DEBUG: ", $fmt), $($arg)*);
    }
}

extern crate fringe;
extern crate spin;

pub mod dependencies;
mod arch;
mod fringe_wrapper;
pub mod scheduler;
pub mod lock;

mod linked_list;
pub mod basic;
