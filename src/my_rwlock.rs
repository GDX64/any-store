use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicI32, Ordering},
};

use crate::extern_functions;

struct InnerValue<T> {
    value: T,
    has_guard: bool,
}

pub struct MyRwLock<T> {
    pub lock: ThreadLock,
    value: UnsafeCell<InnerValue<T>>,
}

unsafe impl<T: Send> Send for MyRwLock<T> {}
unsafe impl<T: Send + Sync> Sync for MyRwLock<T> {}

impl<T> MyRwLock<T> {
    pub fn new(value: T) -> Self {
        MyRwLock {
            lock: ThreadLock::new(),
            value: UnsafeCell::new(InnerValue {
                value,
                has_guard: false,
            }),
        }
    }

    unsafe fn get_mut(&self) -> &mut InnerValue<T> {
        return unsafe { &mut *self.value.get() };
    }

    pub fn write<'a>(&'a self) -> WriteGuard<'a, T> {
        let result = self.lock.lock();
        // SAFETY: We have acquired the lock, so it is safe to access the inner value.
        let inner = unsafe { self.get_mut() };
        // This is protection against reentrant locking
        if inner.has_guard {
            panic!("Guard is already held by this thread");
        }
        inner.has_guard = true;
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
        // SAFETY: We are the owner of the guard, so it is safe to modify the inner value and release the lock.
        let inner = unsafe { self.rwlock.get_mut() };
        inner.has_guard = false;
        if self.result == LockResult::AcquiredFromMe {
            self.rwlock.lock.unlock();
        }
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: There can be only one WriteGuard at a time, so it is safe to access the inner value.
        unsafe { &self.rwlock.get_mut().value }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: There can be only one WriteGuard at a time, so it is safe to access the inner value.
        unsafe { &mut self.rwlock.get_mut().value }
    }
}

const UNLOCKED: i32 = -1;

pub struct ThreadLock {
    lock_state: AtomicI32,
}

#[derive(PartialEq)]
pub enum LockResult {
    AlreadyHeldByCurrentThread,
    AcquiredFromMe,
}

/**
 * This lock work is to guarantee thread access
 * The same thread may acquire the lock multiple times
 * Thus reentrant locking wont cause a deadlock, but may break aliasing rules
 * So the caller must ensure that they dont create multiple guards
 */
impl ThreadLock {
    pub const fn new() -> Self {
        ThreadLock {
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
