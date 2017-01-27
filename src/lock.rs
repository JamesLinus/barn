use core::ops::Deref;
use core::ops::DerefMut;
use core::marker::PhantomData;
use core::cell::UnsafeCell;

use scheduler::{Scheduler, Thread, Request, SchedulerUnit, Queue};

use ::poison::{LockResult, TryLockError, TryLockResult};

pub struct Mutex<T, U: SchedulerUnit> {
  queue_lock: ::spin::Mutex<(U::Q, bool)>,
  data: UnsafeCell<T>,
  p: PhantomData<U>
}

pub struct MutexGuard<'a, T:'a, U: SchedulerUnit> {
    lock: &'a Mutex<T, U>
}

impl<'a, T: 'a, U: SchedulerUnit> MutexGuard<'a, T, U> {

  fn new(lock: &'a Mutex<T, U>) -> MutexGuard<'a, T, U> {
    MutexGuard { lock: lock }
  }

}

impl<'a, T:'a, U: SchedulerUnit> Deref for MutexGuard<'a, T, U> {
  type Target = T;

  fn deref(&self) -> &T {
    unsafe { &*self.lock.data.get() }
  }

}

impl<'a, T:'a, U: SchedulerUnit> DerefMut for MutexGuard<'a, T, U> {
  fn deref_mut(&mut self) -> &mut T {
    unsafe { &mut *self.lock.data.get() }
  }
}

impl<'a, T:'a, U: SchedulerUnit> Drop for MutexGuard<'a, T, U> {

  fn drop(&mut self) {
    self.lock.unlock();
  }

}

impl<T, U: SchedulerUnit> Mutex<T, U> {

  pub fn new(data: T) -> Mutex<T, U> {
    Mutex { queue_lock: ::spin::Mutex::new((U::Q::new(), false)),
            data: UnsafeCell::new(data),
            p: PhantomData::<U>,
    }
  }

  pub fn try_lock(&self) -> TryLockResult<MutexGuard<T, U>> {
    let mut l = self.queue_lock.lock();
    let &mut (_, ref mut taken) = l.deref_mut();
    if *taken {
      Err(TryLockError::WouldBlock)
    } else {
      *taken = true;
      Ok(MutexGuard::new(self))
    }
  }

  pub fn lock(&self) -> LockResult<MutexGuard<T, U>> {
    loop {
      let mut l = self.queue_lock.lock();
      match l.deref_mut() {
        &mut (_, ref mut taken) => {
          if !*taken {
            *taken = true;
            break;
          }
        }
      }
      debug!("didn't get lock, sleeping");
      let take = move |me| {
        match l.deref_mut() {
          &mut (ref mut queue, _) => queue.push(me)
        }
        drop(l);
      };
      Thread::<U>::suspend(Request::make_schedule(&take));
    }
    Ok(MutexGuard::new(self))
  }

  fn unlock(&self) {
    let mut l = self.queue_lock.lock();
    let &mut (ref mut queue, ref mut taken) = l.deref_mut();
    *taken = false;
    if let Some(node) = queue.pop() {
      Thread::<U>::suspend(Request::Schedule(node));
    }
  }
}


unsafe impl<T: Send, U: SchedulerUnit> Send for Mutex<T, U> { }
unsafe impl<T: Send, U: SchedulerUnit> Sync for Mutex<T, U> { }

pub struct Condvar<U: SchedulerUnit> {
  sleepers: ::spin::Mutex<(U::Q)>
}

impl<U: SchedulerUnit> Condvar<U> {

  pub fn new() -> Condvar<U> {
    Condvar { sleepers: ::spin::Mutex::new(U::Q::new()) }
  }

  pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T, U>) -> LockResult<MutexGuard<'a, T, U>> {
    debug!("in wait");
    let mut sleepers = self.sleepers.lock();
    let mutex = guard.lock;
    let take = move |me: U::N| {
      debug!("adding a sleeper");
      sleepers.push(me);
      drop(sleepers);
      drop(guard);
    };
    Thread::<U>::suspend(Request::make_schedule(&take));
    mutex.lock()
  }

  pub fn notify_one(&self) {
    debug!("notifying 1");
    if let Some(node) = self.sleepers.lock().pop() {
      debug!("waking a sleeper");
      Thread::<U>::suspend(Request::Schedule(node));
    }
  }

  pub fn notify_all(&self) {
    let mut sleepers = self.sleepers.lock();
    while let Some(node) = sleepers.pop() {
      Thread::<U>::suspend(Request::Schedule(node));
    }
  }

}


pub struct RwLock<T: ?Sized, U: SchedulerUnit> {
  p: PhantomData<U>,
  __data: UnsafeCell<T>,
}

#[must_use]
pub struct RwLockReadGuard<'a, T: ?Sized + 'a, U: SchedulerUnit> {
  __lock: &'a RwLock<T, U>,
}

impl<'a, T: ?Sized, U: SchedulerUnit> !Send for RwLockReadGuard<'a, T, U> {

}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
#[must_use]
pub struct RwLockWriteGuard<'a, T: ?Sized + 'a, U: SchedulerUnit> {
    __lock: &'a RwLock<T, U>,
}

impl<'a, T: ?Sized, U: SchedulerUnit> !Send for RwLockWriteGuard<'a, T, U> {}

