// Wraps libfringe with some unsafeness to be able
// to use a Yielder outside its function.

use core::prelude::v1::*;

use fringe::{Generator, Stack};
use fringe::generator::Yielder;
use core::intrinsics::transmute;
use core::mem::{forget, uninitialized};

pub struct Group<'a, I, O, S: Stack> where I: Send + 'a, O: Send + 'a {
    generator: Generator<'a, I, O, S>,
    yielder: &'a Yielder<I, O>,
}

impl<'a, I, O, S: Stack> Group<'a, I, O, S> where I: Send + 'a, O: Send + 'a, S: Stack {

    pub fn new<'b, F>(stack: S, f: F) -> Group<'b, I, O, S> where F: FnOnce() + Send + 'b {
        // Alternative to the raw pointers and zero(.) is to pass `yielder` with
        // .suspend(.). That still requires a transmute due to life-times. Not worth
        // the hassel...
        unsafe {
          let yielder_ptr: &mut usize = &mut 0;
          debug!("yielder ptr: 0x{:x}", yielder_ptr as *const usize as usize);
          let yielder_usize: usize = yielder_ptr as *const usize as usize;
          let mut gen = Generator::unsafe_new(stack, move |yielder, init| {
              forget(init);
              debug!("gen start: t 0x{:x}", transmute::<_, usize>(yielder_usize));
              *transmute::<_, *mut usize>(yielder_usize) = transmute(yielder);
              debug!("inner yielder at 0x{:x}", *(yielder_usize as *const usize));
              debug!("bar {}", 1);
              yielder.suspend(uninitialized());
              f();
          });

          forget(gen.resume(uninitialized()));
          debug!("got yielder at 0x{:x}", *yielder_ptr);
          Group { generator: gen, yielder: transmute(*yielder_ptr)}
        }
    }
    
    // Unsafe because needs to be called in the right thread...
    pub unsafe fn resume(&mut self, i: I) -> Option<O> {
        self.generator.resume(i)        
    }
    
    // Unsafe because needs to be called in the right thread...
    pub unsafe fn suspend(&self, o: O) -> I {
        //info!("suspending to yielder at 0x{:x}", self.yielder as *const Yielder<_, _> as usize);
        self.yielder.suspend(o)
    }

}