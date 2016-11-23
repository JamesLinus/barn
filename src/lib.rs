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

extern crate fringe;
extern crate spin;

mod arch;
mod fringe_wrapper;
pub mod scheduler;

mod linked_list;
pub mod basic;
