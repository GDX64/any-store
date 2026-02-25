use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicI32, Ordering},
};

use crate::extern_functions;

/**
 * IMPORTANT
 * I am ignoring the lock now because of a race condition I could not find yet
 * Because of this race condition, I am already locking the whole wasm module before calling the funcions
 */
pub struct MyRwLock<T> {
    lock: Lock,
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
        self.lock.lock();
        return WriteGuard { rwlock: &self };
    }
}

pub struct WriteGuard<'a, T> {
    rwlock: &'a MyRwLock<T>,
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.rwlock.lock.unlock();
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

const LOCKED: i32 = 1;
const UNLOCKED: i32 = 0;

pub struct Lock {
    is_locked: AtomicI32,
}

impl Lock {
    pub const fn new() -> Self {
        Lock {
            is_locked: AtomicI32::new(UNLOCKED),
        }
    }

    pub fn lock(&self) {
        while self
            .is_locked
            .compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            if extern_functions::is_main_thread() {
                continue;
            }
            #[cfg(target_arch = "wasm32")]
            unsafe {
                let ptr = self.is_locked.as_ptr();
                std::arch::wasm32::memory_atomic_wait32(ptr, 1, 1000_000);
            }
        }
    }

    pub fn unlock(&self) {
        self.is_locked.store(UNLOCKED, Ordering::Release);
        #[cfg(target_arch = "wasm32")]
        unsafe {
            std::arch::wasm32::memory_atomic_notify(self.is_locked.as_ptr(), 1);
        }
    }
}
