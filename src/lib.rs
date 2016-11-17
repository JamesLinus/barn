#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(const_fn)]
#![no_std]

#[cfg(test)]
extern crate std;

extern crate fringe;

pub mod scheduler;
mod fringe_wrapper;
