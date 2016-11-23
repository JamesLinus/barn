// Variable local to scheduler

use core::mem::transmute;

pub struct Guard {
  result: local_impl::Result
}

impl Drop for Guard {

  fn drop(&mut self) {
    no_preempt_end(self.result);
  }

}

fn no_preempt_start() -> local_impl::Result {
  local_impl::no_preempt_start()
}

fn no_preempt_end(result: local_impl::Result) {
  local_impl::no_preempt_end(result)
}


pub struct Arch<T> {
  p: ::core::marker::PhantomData<T>
}

impl<T> Arch<T> {

  pub unsafe fn no_preempt() -> Guard {
    Guard { result: no_preempt_start() }
  }


  pub unsafe fn get() -> &'static mut T {
    unsafe { transmute(local_impl::get()) }
  }
  
  pub unsafe fn set(value: &'static T) {
    local_impl::set(value as *const T as usize)
  }

}

// TODO: make for each arch, this is only for
// a mod with std
mod local_impl {

  use std::cell::RefCell;

  pub type Result = bool;

  thread_local! {
    pub static LOCAL: RefCell<usize> = RefCell::new(0);
    pub static NO_PREEMPT: RefCell<Result> = RefCell::new(false);
  }
  
  pub fn get() -> usize {
    let mut ret = 0;
    LOCAL.with(|l| ret = *l.borrow());
    ret
  }
  
  pub fn set(value: usize) {
    LOCAL.with(|l| *l.borrow_mut() = value);
  }

  pub fn no_preempt_start() -> Result {
    // TODO: should be atomic bool...
    let mut old = false;
    NO_PREEMPT.with(|l| { old = *l.borrow(); *l.borrow_mut() = true });
    old
  }

  pub fn no_preempt_end(old: Result) {
    NO_PREEMPT.with(|l| { *l.borrow_mut() = old });
  }

}
