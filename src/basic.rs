
extern crate alloc;

use self::alloc::boxed::Box;

use fringe::OwnedStack;
use scheduler;
use lock;

use core::ops::Deref;

pub struct Unit;
impl ::scheduler::SchedulerUnit for Unit {
  type L = Local;
  type N = Node;
  type Q = Queue;
  type S = OwnedStack;
}

type Local = Option<()>;

pub type Node = Box<::linked_list::Node<scheduler::Thread<Unit>>>;
pub type Queue = ::linked_list::LinkedList<scheduler::Thread<Unit>>;
pub type Scheduler = scheduler::Scheduler<Unit>;
pub type Mutex<T> = lock::Mutex<T, Unit>;
pub type MutexGuard<'a, T> = lock::MutexGuard<'a, T, Unit>;
pub type Condvar = lock::Condvar<Unit>;
pub type RwLock<T> =  lock::RwLock<T, Unit>;
pub type RwLockReadGuard<'a, T> = lock::RwLockReadGuard<'a, T, Unit>;
pub type RwLockWriteGuard<'a, T> = lock::RwLockWriteGuard<'a, T, Unit>;
pub type Thread = scheduler::Thread<Unit>;


impl scheduler::Node<Unit> for Node {

  fn new(t: Thread) -> Self {
    box ::linked_list::Node::new(t)
  }

  fn deref(&self) -> &Thread {
    &self.value
  }

  fn deref_mut(&mut self) -> &mut Thread {
    &mut self.value
  }

}

impl ::scheduler::Queue<Unit> for Queue {

  fn new() -> Queue {
    ::linked_list::LinkedList::new()
  }

  fn push(&mut self, node: Node) {
    self.push_back_node(node);
  }
  
  fn pop(&mut self) -> Option<Node> {
    self.pop_front_node()
  }
  
  fn front(&self) -> Option<&Node> {
    self.list_head.as_ref()
  }

  fn front_mut(&mut self) -> Option<&mut Node> {
    self.list_head.as_mut()
  }

}

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}


#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use super::*;
  use scheduler::Request;
  use fringe::{OwnedStack, Generator};

  fn thread<F: FnOnce() + Send + 'static>(f: F) -> Thread {
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
    debug!("pushed: 0x{:x}", q.front().unwrap() as *const Thread as usize);

    let mut s: Scheduler = Scheduler::new(q);
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
      Thread::suspend(Request::Yield);
    });
  }


  #[test]
  fn mutex_smoke() {
    let mut q = Queue::new();
    let ran = Arc::new(Mutex::new(false));
    let saved_ran = ran.clone();
    let t = thread(move || {
      *ran.lock().unwrap() = true;
    });
    q.push_front(t);
    let mut s: Scheduler = Scheduler::new(q);
    s.run();
    assert!(*saved_ran.lock().unwrap());
  }

  #[test]
  fn mutex_contention() {
    let mut q = Queue::new();
    let sum = Arc::new(Mutex::new(0));
    let sum_copy1 = sum.clone();
    let sum_copy2 = sum.clone();

    let t = thread(move || {
      let mut i = sum_copy1.lock().unwrap();
      Thread::suspend(Request::Yield);
      // t runs first so it gets the lock first
      assert!(*i == 0);
      *i += 1;
    });

    let t2 = thread(move || {
      *sum_copy2.lock().unwrap() += 1;
    });

    q.push_front(t2);
    q.push_front(t);
    let mut s: Scheduler = Scheduler::new(q);
    s.run();
    assert!(*sum.lock().unwrap() == 2);
  }

  #[test]
  fn condvar_smoke() {
    let mut q = Queue::new();

    let pair = Arc::new((Mutex::new(vec!()), Condvar::new()));
    let pair2 = pair.clone();
    let pair3 = pair.clone();

    let t1 = thread(move || {
      let inner = thread(move || {
        let &(ref lock, ref cvar) = &*pair2;
        let mut v = lock.lock().unwrap();
        while v.len() == 0 {
          v = cvar.wait(v).unwrap();
        }
        v.push(2);
        cvar.notify_one();
      });

      Thread::suspend(Request::Schedule(Node::new(inner)));

      let &(ref lock, ref cvar) = &*pair;
      let mut v = lock.lock().unwrap();
      v.push(1);
      v = cvar.wait(v).unwrap();
      v.push(3)
    });

    q.push_front(t1);

    Scheduler::new(q).run();
    let &(ref lock, _) = &*pair3;
    assert_eq!(*lock.lock().unwrap(), vec!(1, 2, 3));
  }

}
