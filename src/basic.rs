
extern crate alloc;

use self::alloc::boxed::Box;

use fringe::OwnedStack;
use scheduler::{Thread, SchedulerUnit};

struct Unit;
impl SchedulerUnit for Unit {
  type L = Local;
  type N = Node;
  type Q = Queue;
  type S = OwnedStack;
}

type Local = Option<()>;

type Node = Box<::linked_list::Node<Thread<Unit>>>;
type Queue = ::linked_list::LinkedList<Thread<Unit>>;

impl ::scheduler::Node<Unit> for Node {

  fn deref(&self) -> &Thread<Unit> {
    &self.value
  }
  
  fn deref_mut(&mut self) -> &mut Thread<Unit> {
    &mut self.value
  }

}

impl ::scheduler::Queue<Unit> for Queue {

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
  use std::sync::{Arc, Mutex};

  use super::{Queue, Unit};
  use scheduler::{Scheduler, Thread};
  use fringe::{OwnedStack, Generator};

  #[test]
  fn threads() {
    let mut q = Queue::new();
    let stack = OwnedStack::new(1024 * 1024);
    let ran = Arc::new(Mutex::new(false));
    let saved_ran = ran.clone();
    let g = unsafe {
      Thread::new(stack, move || {
      *ran.lock().unwrap() = true;
      })
    };
    q.push_front(g);
    let mut s: Scheduler<Unit> = Scheduler::new(q);
    s.run();
    assert!(*saved_ran.lock().unwrap());
  }
}
