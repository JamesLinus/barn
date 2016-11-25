
extern crate alloc;

use self::alloc::boxed::Box;

use fringe::OwnedStack;
use scheduler::{Thread, Node};

use core::ops::Deref;

struct Unit;
impl ::scheduler::SchedulerUnit for Unit {
  type L = Local;
  type N = BasicNode;
  type Q = Queue;
  type S = OwnedStack;
}

type Local = Option<()>;

type BasicNode = Box<::linked_list::Node<Thread<Unit>>>;
type Queue = ::linked_list::LinkedList<Thread<Unit>>;

impl Node<Unit> for BasicNode {

  fn deref(&self) -> &Thread<Unit> {
    &self.value
  }

  fn deref_mut(&mut self) -> &mut Thread<Unit> {
    &mut self.value
  }

}

impl ::scheduler::Queue<Unit> for Queue {

  fn new() -> Queue {
    ::linked_list::LinkedList::new()
  }

  fn push(&mut self, node: BasicNode) {
    self.push_back_node(node);
  }
  
  fn pop(&mut self) -> Option<BasicNode> {
    self.pop_front_node()
  }
  
  fn front(&self) -> Option<&BasicNode> {
    self.list_head.as_ref()
  }

  fn front_mut(&mut self) -> Option<&mut BasicNode> {
    self.list_head.as_mut()
  }

}

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use super::{Queue, Unit};
  use scheduler::{Scheduler, Thread, Request};
  use fringe::{OwnedStack, Generator};
  use lock::{Mutex};

  fn thread<F: FnOnce() + Send + 'static>(f: F) -> Thread<Unit> {
    let stack = OwnedStack::new(1024 * 1024);
    unsafe { Thread::new(stack, f) }
  }

  fn smoke<F: FnOnce() + Send + 'static>(f: F) {
    let mut q = Queue::new();
    let ran = Arc::new(::std::sync::Mutex::new(false));
    let saved_ran = ran.clone();
    let t = thread(move || {
      f();
      *ran.lock().unwrap() = true;
    });
    q.push_front(t);
    debug!("pushed: 0x{:x}", q.front().unwrap() as *const Thread<Unit> as usize);

    let mut s: Scheduler<Unit> = Scheduler::new(q);
    s.run();
    assert!(*saved_ran.lock().unwrap());
  }

  #[test]
  fn thread_smoke() {
    smoke(|| {});
  }

  #[test]
  fn yield_smoke() {
    smoke(|| {
      Thread::<Unit>::suspend(Request::Yield);
    });
  }


  #[test]
  fn mutex_smoke() {
    let mut q = Queue::new();
    let ran = Arc::new(Mutex::<_, Unit>::new(false));
    let saved_ran = ran.clone();
    let t = thread(move || {
      *ran.lock() = true;
    });
    q.push_front(t);
    let mut s: Scheduler<Unit> = Scheduler::new(q);
    s.run();
    assert!(*saved_ran.lock());
  }

  #[test]
  fn mutex_contention() {
    let mut q = Queue::new();
    let sum = Arc::new(Mutex::<_, Unit>::new(0));
    let sum_copy1 = sum.clone();
    let sum_copy2 = sum.clone();

    let t = thread(move || {
      let mut i = sum_copy1.lock();
      Thread::<Unit>::suspend(Request::Yield);
      // t runs first so it gets the lock first
      assert!(*i == 0);
      *i += 1;
    });

    let t2 = thread(move || {
      *sum_copy2.lock() += 1;
    });

    q.push_front(t2);
    q.push_front(t);
    let mut s: Scheduler<Unit> = Scheduler::new(q);
    s.run();
    assert!(*sum.lock() == 2);
  }

}
