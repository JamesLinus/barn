#![allow(dead_code)]

use core::mem::{transmute};

use fringe_wrapper::Group;

pub trait SchedulerUnit where Self: Sized + 'static {
  type L: Default;
  type Q: Queue<Self>;
  type N: Node<Self>;
  type S: ::fringe::Stack;
}

pub trait Node<U: SchedulerUnit> where Self: Send + Sized {
  // TODO: should be able to inhereit associated type, but looks like compiler problem.
  fn deref(&self) -> &Thread<U>;

  fn deref_mut(&mut self) -> &mut Thread<U>;
}

// Must do no allocations for these methods
// Invariant is that current thread is at the front of the queue.
pub trait Queue<U: SchedulerUnit> where Self: Sized + Sync + 'static {

  fn new() -> Self;

  fn push(&mut self, node: U::N);

  fn pop(&mut self) -> Option<U::N>;

  fn front(&self) -> Option<&U::N>;

  fn front_mut(&mut self) -> Option<&mut U::N>;
}


pub struct Thread<U: SchedulerUnit> {
  group: Group<'static, Response<U>, Request<U>, U::S>,
  local: U::L
}

type Arch<U: SchedulerUnit> = ::arch::Arch<(Thread<U>)>;

impl<U: SchedulerUnit> Thread<U> {

  pub fn new<F>(stack: U::S, f: F) -> Thread<U> where F: FnOnce() + Send + Sized + 'static {
    Thread {
      group: Group::new(stack, f),
      local: U::L::default(),
    }
  }

  pub fn suspend(request: Request<U>) -> Response<U> {
    let _guard = unsafe { Arch::<U>::no_preempt() };// no interrupts while switching
    let me = Self::current();
    unsafe { me.group.suspend(request) }
  }

  fn resume(&mut self, response: Response<U>) -> Option<Request<U>> {
    // Make sure to set the thread locals on resume.
    // This doesn't go in suspend because it needs to also be set on
    // thread start.
    unsafe {
      let me: &'static Self = transmute(&self);
      Arch::<U>::set(me);
      self.group.resume(response)
    }
  }

  pub fn current() -> &'static Thread<U> {
    unsafe { Arch::<U>::get() }
  }

  pub fn current_mut() -> &'static mut Thread<U> {
    unsafe { Arch::<U>::get() }
  }

  pub fn local(&self) -> &U::L {
    &self.local
  }

  pub fn local_mut(&mut self) -> &mut U::L {
    &mut self.local
  }

}

unsafe impl<U: SchedulerUnit> Send for Group<'static, Response<U>, Request<U>, U::S> {}

pub enum Request<U: SchedulerUnit> {
    Yield,
    Schedule(U::N),
    Unschedule(Option<&'static (Fn(U::N) -> () + Sync)>),
}

impl<U: SchedulerUnit> Request<U> {

  pub fn make_schedule(use_node: &(FnOnce(U::N) -> ())) -> Request<U> {
    // safe because the function is not used after it is called once
    let f = unsafe { ::core::mem::transmute(use_node) };
    Request::Unschedule(Some(f))
  }

}

pub enum Response<U: SchedulerUnit> {
    Nothing,
    Unscheduled(Option<U::N>)
}

pub struct Scheduler<U: SchedulerUnit> {
    queue: U::Q,
}

impl<U: SchedulerUnit> Scheduler<U> {
  
  // Creates a scheduler with the given thread queue
  pub fn new(queue: U::Q) -> Scheduler<U> {
    Scheduler { queue: queue }
  }
  
  fn next_request(&mut self, response: Response<U>) -> Option<Request<U>> {
    let front: Option<&mut U::N> = self.queue.front_mut();
    front.map(|x| x.deref_mut().resume(response).unwrap_or(Request::Unschedule(None)))
  }

  pub fn run(&mut self) {
    let mut response = Response::Nothing;
    
    while let Some(request) = self.next_request(response) {
        response = match request {
          Request::Yield => {
              let c = self.queue.pop().unwrap();
              self.queue.push(c);
              Response::Nothing
          },
          Request::Unschedule(maybe_taker) => {
            let node = self.queue.pop().unwrap();
            match maybe_taker {
              Some(ref taker) => {
                taker(node);
                Response::Unscheduled(None)
              }
              None => Response::Unscheduled(Some(node))
            }
          },
          Request::Schedule(tcb_node) => {
              self.queue.push(tcb_node);
              Response::Nothing
          },
        }
    }
  }

}
