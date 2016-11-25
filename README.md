libbarn
=====

A (work in progress) bare-metal scheduler.

### Design
No dependencies on `std` or `alloc` unless `hosted` feature is enabled.

Scheduling algorithm and datastructures are stubbed out as traits (see `basic.rs` for a
simple implementation). `barn` only provides primitives to manage threads (including
locking) and interact with the scheduler.
