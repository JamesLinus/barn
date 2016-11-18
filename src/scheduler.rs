use core::mem::{transmute, transmute_copy};
use core::ptr;

use fringe::SliceStack;

use super::fringe_wrapper::Group;

type G<U: Unit> = Group<'static, ThreadResponse<U>, ThreadRequest<U>, U::S>;

pub struct Thread<U: Unit> {
  group: G<U>,
  name: &'static str,
  local: U::L,
}

unsafe impl<U: Unit> Send for Thread<U> {}

impl<U: Unit> ::core::ops::Deref for Thread<U> {
    type Target = U::L;

    fn deref(&self) -> &U::L {
        &self.local
    }
}



impl<U: Unit> Thread<U> {
  
  fn new<F>(f: F, stack: U::S, name: &'static str) -> Thread<U>  where F: FnOnce() + Send + 'static {
    Thread {
      group: unsafe { Group::new(f, stack) },
      name: name,
      local: U::L::default(),
    }
  }
  
  unsafe fn request(&mut self, request: ThreadRequest<U>) -> ThreadResponse<U> {
    self.group.suspend(request)
  }

}

pub trait Node<U: Unit> where Self: Send + Sized {
  
  fn new(t: Thread<U>) -> Self where Self: Sized;
  
  // TODO: should be able to inhereit associated type, but looks like compiler problem.
  fn deref(&self) -> &Thread<U>;
  
  fn deref_mut(&mut self) -> &mut Thread<U>;
}

pub trait Stack: ::fringe::Stack + Sized {
  fn new(size: u8) -> Self;
}

// Must do no allocations for these methods
pub trait Queue<U: Unit> where Self: Sized + Sync + 'static {
  
  fn push(&mut self, node: U::N);
  
  fn pop(&mut self) -> Option<U::N>;
  
  fn front(&self) -> Option<&U::N>;

  fn front_mut(&mut self) -> Option<&mut U::N>;
}

pub trait Unit: 'static + Sized + Sync {
  type L: Default;
  type S: Stack;
  type N: Node<Self>;
}

// Request of thread to scheduler
enum ThreadRequest<U: Unit> {
    Yield,
    StageUnschedule,
    Schedule(U::N),
    CompleteUnschedule,
}

// Response
enum ThreadResponse<U: Unit> {
    Nothing,
    Unscheduled(U::N)
}

pub struct Scheduler<U: Unit, Q: Queue<U>> {
    queue: Q,
    _phantom: ::core::marker::PhantomData<U>, // complains the U is not used but is used for type arg of Q...
}

impl<U: Unit, Q: Queue<U>> Scheduler<U, Q> {
  
  // Creates a scheduler with the given thread queue
  fn new(queue: Q) -> Scheduler<U, Q> {
    Scheduler { queue: queue, _phantom: ::core::marker::PhantomData }
  }
  
  fn _idle_thread(&self) -> Thread<U> {
    unsafe {
      let me: usize = ::core::mem::transmute(self);      
      Thread::new(move || {
        let me_static: &'static mut Scheduler<U, Q> = ::core::mem::transmute(me);
        me_static.idle() }, 
        U::S::new(1024*1024), "idle thread")
    }
  }
  
  fn current_thread(&self) -> &Thread<U> {
    self.queue.front().unwrap().deref()
  }
  
  fn current_thread_mut(&mut self) -> &mut Thread<U> {
    self.queue.front_mut().unwrap().deref_mut()
  }
  
  unsafe fn run(&mut self) -> ! {
    // scheduler now takes control of the CPU
    let mut idle = self._idle_thread();
    self.queue.push(U::N::new(idle));
    
    let mut response = ThreadResponse::Nothing;
    loop {
        let mut request = self.queue.front_mut().unwrap().deref_mut().group.resume(response);
        response = match request {
            Some(req) => match req {
                ThreadRequest::Yield => {
                    let c = self.queue.pop().unwrap();
                    self.queue.push(c);
                    ThreadResponse::Nothing
                },
                ThreadRequest::StageUnschedule => {
                    // We have to pass `node` to a resume call on the tcb in node.
                    // To do so, we need to get around the borrow checker.
                    let mut node: &U::N = self.queue.front().unwrap();
                    let static_node: *const U::N = unsafe { ::core::mem::transmute(node) };                        
                    unsafe { ThreadResponse::Unscheduled(::core::ptr::read(static_node)) }
                },
                ThreadRequest::CompleteUnschedule => {
                    // We can assert that last response was unscheduled
                    // Finish unscheduling. Thread's ownership has already been passed
                    ::core::mem::forget(self.queue.pop());
                    ThreadResponse::Nothing
                },
                ThreadRequest::Schedule(tcb_node) => {
                    self.queue.push(tcb_node);
                    ThreadResponse::Nothing
                },
            },
            None => {
                // Thread is finished.
                if let Some(node) = self.queue.pop() {
                  ThreadResponse::Unscheduled(node) //don't drop in the scheduler!
                } else {
                  ThreadResponse::Nothing
                }
            },
        }
    }
  }
  
  fn idle(&mut self) {
    loop {
        unsafe { self.current_thread_mut().request(ThreadRequest::Yield); }
    }
  }

}
