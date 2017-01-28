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

#[cfg(feature = "hosted")]
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

#[cfg(all(not(feature = "hosted"), target_arch = "x86"))]
mod local_impl {

  pub type Result = bool;

  pub fn get() -> usize {
    let gs: u32;
    unsafe {
      asm!("movl %dr0, $0"
          :"={eax}"(gs)
          :
          :
          :"volatile");
    }
    gs as usize
  }

  pub fn set(value: usize) {
    unsafe {
      asm!("movl $0, %dr0"
          :
          :"{eax}"(value as u32)
          :
          :"volatile");
    }
  }

  unsafe fn interrupts_enabled() -> bool {
    let eflags: u32;
    asm!("pushfd\n
        popl %eax"
        :"={eax}"(eflags)
        :
        :
        :"volatile");
    (eflags >> 9 & 1) == 1
  }


  pub fn no_preempt_start() -> Result {
    unsafe {
      let was_enabled: bool = interrupts_enabled();
      asm!("cli" :::: "volatile");
      was_enabled
    }
  }

  pub fn no_preempt_end(was_enabled: Result) {
    if was_enabled {
      unsafe { asm!("sti" :::: "volatile"); }
    }
  }
}
