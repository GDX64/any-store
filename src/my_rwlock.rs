use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

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

    pub fn read<'a>(&'a self) -> ReadGuard<'a, T> {
        self.lock.lock();
        return ReadGuard { rwlock: &self };
    }

    pub fn write<'a>(&'a self) -> WriteGuard<'a, T> {
        self.lock.lock();
        return WriteGuard { rwlock: &self };
    }
}

pub struct ReadGuard<'a, T> {
    rwlock: &'a MyRwLock<T>,
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        self.rwlock.lock.unlock();
    }
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.rwlock.value.get() }
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

const LOCKED: bool = true;
const UNLOCKED: bool = false;

pub struct Lock {
    is_locked: AtomicBool,
}

impl Lock {
    pub const fn new() -> Self {
        Lock {
            is_locked: AtomicBool::new(UNLOCKED),
        }
    }

    pub fn lock(&self) {
        while self
            .is_locked
            .compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        self.is_locked.store(UNLOCKED, Ordering::Release);
    }
}
