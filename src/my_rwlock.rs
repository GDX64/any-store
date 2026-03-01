use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicI32, Ordering},
};

use crate::extern_functions;

pub struct MyRwLock<T> {
    pub lock: Lock,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for MyRwLock<T> {}
unsafe impl<T: Send + Sync> Sync for MyRwLock<T> {}

impl<T> MyRwLock<T> {
    pub fn new(value: T) -> Self {
        MyRwLock {
            lock: Lock::new(),
            value: UnsafeCell::new(value),
        }
    }

    pub fn write<'a>(&'a self) -> WriteGuard<'a, T> {
        let result = self.lock.lock();
        return WriteGuard {
            rwlock: &self,
            result,
        };
    }
}

pub struct WriteGuard<'a, T> {
    rwlock: &'a MyRwLock<T>,
    result: LockResult,
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        if self.result == LockResult::AcquiredFromMe {
            self.rwlock.lock.unlock();
        }
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.rwlock.value.get() }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.rwlock.value.get() }
    }
}

const UNLOCKED: i32 = -1;

pub struct Lock {
    lock_state: AtomicI32,
}

#[derive(PartialEq)]
pub enum LockResult {
    AlreadyHeldByCurrentThread,
    AcquiredFromMe,
}

impl Lock {
    pub const fn new() -> Self {
        Lock {
            lock_state: AtomicI32::new(UNLOCKED),
        }
    }

    pub fn lock(&self) -> LockResult {
        let state = self.lock_state.load(Ordering::Relaxed);
        if state == current_thread_value() {
            return LockResult::AlreadyHeldByCurrentThread;
        }

        while self
            .lock_state
            .compare_exchange(
                UNLOCKED,
                current_thread_value(),
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            if extern_functions::is_main_thread() {
                continue;
            }
            #[cfg(target_arch = "wasm32")]
            unsafe {
                let ptr = self.lock_state.as_ptr();
                std::arch::wasm32::memory_atomic_wait32(ptr, 1, 1000_000);
            }
        }
        return LockResult::AcquiredFromMe;
    }

    pub fn unlock(&self) {
        self.lock_state.store(UNLOCKED, Ordering::Release);
        #[cfg(target_arch = "wasm32")]
        unsafe {
            std::arch::wasm32::memory_atomic_notify(self.lock_state.as_ptr(), 1);
        }
    }

    pub fn try_lock(&self) -> bool {
        return self
            .lock_state
            .compare_exchange(
                UNLOCKED,
                current_thread_value(),
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok();
    }

    pub fn pointer(&self) -> *const i32 {
        return self.lock_state.as_ptr();
    }
}

fn current_thread_value() -> i32 {
    return extern_functions::worker_id() as i32;
}
