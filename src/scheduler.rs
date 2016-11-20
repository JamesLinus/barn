#![allow(dead_code)]

use core::mem::{transmute};

pub type Generator<U: SchedulerUnit> = ::fringe::Generator<'static, Response<U>, Request<U>, U::S>;
pub type Yielder<U: SchedulerUnit> = ::fringe::generator::Yielder<Response<U>, Request<U>>;

pub trait SchedulerUnit where Self: Sized + 'static {
  type Q: Queue<Self>;
  type N: Node<Self>;
  type S: ::fringe::Stack;
}

pub trait Node<U: SchedulerUnit> where Self: Send + Sized {
  
  // TODO: should be able to inhereit associated type, but looks like compiler problem.
  fn deref(&self) -> &Generator<U>;
  
  fn deref_mut(&mut self) -> &mut Generator<U>;
}

// Must do no allocations for these methods
// Invariant is that current thread is at the front of the queue.
pub trait Queue<U: SchedulerUnit> where Self: Sized + Sync + 'static {
  
  fn push(&mut self, node: U::N);
  
  fn pop(&mut self) -> Option<U::N>;
  
  fn front(&self) -> Option<&U::N>;

  fn front_mut(&mut self) -> Option<&mut U::N>;
}

pub enum Request<U: SchedulerUnit> {
    Yield,
    Schedule(U::N),
    Unschedule(Option<&'static (Fn(U::N) -> () + Sync)>),
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
              Some(taker) => {
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
