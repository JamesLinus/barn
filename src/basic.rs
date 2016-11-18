extern crate alloc;

use fringe::OwnedStack;
use scheduler::Thread;

type Node = alloc::boxed::Box<::linked_list::Node<Thread<U>>>;
type Queue = ::linked_list::LinkedList<Thread<U>>;
type Stack = OwnedStack;

struct U;
impl ::scheduler::Unit for U {
  type L = Local;
  type S = Stack;
  type N = Node;
}

pub struct Local {

}

impl Default for Local {

  fn default() -> Local {
    Local {}
  }

}

impl ::scheduler::Stack for Stack {
  
  fn new(size: usize) -> Self {
    OwnedStack::new(size)
  }
  
}

impl ::scheduler::Node<U> for Node {

  fn new(t: Thread<U>) -> Node {
    box ::linked_list::Node::new(t)
  }

  fn deref(&self) -> &Thread<U> {
    &self.value
  }
  
  fn deref_mut(&mut self) -> &mut Thread<U> {
    &mut self.value
  }

}


impl ::scheduler::Queue<U> for Queue {

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
  use fringe::OwnedStack;

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