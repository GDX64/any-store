use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct MyRwLock<T> {
    is_locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for MyRwLock<T> {}
unsafe impl<T: Send + Sync> Sync for MyRwLock<T> {}

impl<T> MyRwLock<T> {
    pub fn new(value: T) -> Self {
        MyRwLock {
            is_locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn read<'a>(&'a self) -> ReadGuard<'a, T> {
        loop {
            let was_free = self.is_locked.swap(true, Ordering::Acquire);
            if was_free {
                return ReadGuard { rwlock: &self };
            } else {
                std::hint::spin_loop();
            }
        }
    }

    pub fn write<'a>(&'a self) -> WriteGuard<'a, T> {
        loop {
            let was_free = self.is_locked.swap(true, Ordering::Acquire);
            if was_free {
                return WriteGuard { rwlock: &self };
            } else {
                std::hint::spin_loop();
            }
        }
    }
}

pub struct ReadGuard<'a, T> {
    rwlock: &'a MyRwLock<T>,
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        self.rwlock.is_locked.store(false, Ordering::Release);
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
        self.rwlock.is_locked.store(false, Ordering::Release);
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
