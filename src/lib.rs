#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(const_fn)]
#![no_std]
#![feature(alloc)]
#![feature(rand)]
#![feature(associated_type_defaults)]

#[cfg(test)]
#[macro_use]
extern crate std;

extern crate fringe;

pub mod scheduler;

mod fringe_wrapper;

mod linked_list;
pub mod basic;
