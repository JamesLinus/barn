/*
extern crate alloc;

use self::alloc::boxed::Box;

use fringe::OwnedStack;
use scheduler::{Generator, Yielder, SchedulerUnti};

struct Unit;
impl SchedulerUnit for Unit {
  type N = Node;
  type Q = Queue;
  type 
}

type Node = Box<::linked_list::Node<Generator<Node, OwnedStack>>>;
type Queue = ::linked_list::LinkedList<Gen>;

impl ::scheduler::Node for Node {

  fn deref(&self) -> &Generator<Node, OwnedStack> {
    &self.value
  }
  
  fn deref_mut(&mut self) -> &mut Generator<Node, OwnedStack> {
    &mut self.value
  }

}

impl ::scheduler::Queue for Queue {

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

  use super::{Queue};
  use scheduler::{Scheduler, Thread};
  use fringe::{OwnedStack, Generator, Yielder};

  #[test]
  fn threads() {
    let mut q = Queue::new();
    let stack = OwnedStack::new(1024 * 1024);
    let ran = Arc::new(Mutex::new(false));
    let saved_ran = ran.clone();
    let f = move || {
      *ran.lock().unwrap() = true;
    };
    q.push_front(Thread::new(f, stack, "test"));
    let mut s = Scheduler::new(q);
    s.run();
    assert!(*saved_ran.lock().unwrap());
  }
}
*/