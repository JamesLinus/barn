#![feature(reflect_marker)]
#![feature(optin_builtin_traits)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(const_fn)]
#![no_std]
#![feature(alloc)]
#![feature(rand)]
#![feature(associated_type_defaults)]
#![feature(asm)] 

#[cfg(any(test, feature = "hosted"))]
#[macro_use]
extern crate std;

#[cfg(test)]
macro_rules! debug {
    ($fmt:expr) => {
      println!(concat!("DEBUG: ", $fmt));
    };
    ($fmt:expr, $($arg:tt)*) => {
      println!(concat!("DEBUG: ", $fmt), $($arg)*);
    }
}

#[cfg(not(test))]
macro_rules! debug {
    ($fmt:expr) => {
    };
    ($fmt:expr, $($arg:tt)*) => {
    }
}

extern crate fringe;
extern crate spin;

mod arch;

mod fringe_wrapper;

pub mod scheduler;

pub mod lock;

mod linked_list;
pub mod basic;
pub mod poison;
