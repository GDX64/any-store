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

thread_local! {
    static HAS_LOCK: UnsafeCell<bool> = UnsafeCell::new(false);
}

fn has_global_lock() -> bool {
    return HAS_LOCK.with(|v| unsafe { *v.get() });
}

fn set_global_lock(value: bool) {
    HAS_LOCK.with(|v| unsafe { *v.get() = value });
}

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
        self.lock.lock_write();
        // SAFETY: We have acquired the lock, so it is safe to access the inner value.
        let inner = unsafe { self.get_mut() };
        // This is protection against reentrant locking
        if inner.has_guard {
            panic!("Guard is already held by this thread");
        }
        inner.has_guard = true;
        return WriteGuard { rwlock: &self };
    }
}

pub struct WriteGuard<'a, T> {
    rwlock: &'a MyRwLock<T>,
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: We are the owner of the guard, so it is safe to modify the inner value and release the lock.
        let inner = unsafe { self.rwlock.get_mut() };
        inner.has_guard = false;
        self.rwlock.lock.release_write();
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

const UNLOCKED: i32 = 0;
const WRITE: i32 = -1;

pub struct ThreadLock {
    lock_state: AtomicI32,
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

    fn lock_read(&self) {
        if has_global_lock() {
            return;
        }
        loop {
            let state = self.lock_state.load(Ordering::Relaxed);
            if !has_writer(state) {
                let is_ok = self
                    .lock_state
                    .compare_exchange(state, state + 1, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok();
                if is_ok {
                    return;
                }
            }
            wait(&self.lock_state);
        }
    }

    fn lock_write(&self) {
        if has_global_lock() {
            return;
        }
        loop {
            let state = self.lock_state.load(Ordering::Relaxed);
            if is_unlocked(state) {
                let is_ok = self
                    .lock_state
                    .compare_exchange(state, WRITE, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok();
                if is_ok {
                    return;
                }
            }
            wait(&self.lock_state);
        }
    }

    fn release_read(&self) {
        if has_global_lock() {
            return;
        }
        self.lock_state.fetch_sub(-1, Ordering::Release);
        notify(&self.lock_state);
    }

    fn release_write(&self) {
        if has_global_lock() {
            return;
        }
        self.lock_state.store(UNLOCKED, Ordering::Release);
        notify(&self.lock_state);
    }

    pub fn try_global_lock_write(&self) -> bool {
        if has_global_lock() {
            panic!("Global lock is already held by this thread");
        }
        let ok = self
            .lock_state
            .compare_exchange(UNLOCKED, WRITE, Ordering::Acquire, Ordering::Relaxed)
            .is_ok();
        if ok {
            set_global_lock(true);
        }
        return ok;
    }

    pub fn global_lock_write(&self) {
        if has_global_lock() {
            panic!("Global lock is already held by this thread");
        }
        self.lock_write();
        set_global_lock(true);
    }

    pub fn release_global_write(&self) {
        if !has_global_lock() {
            panic!("Global lock is not held by this thread");
        }
        self.lock_state.store(UNLOCKED, Ordering::Release);
        notify(&self.lock_state);
        set_global_lock(false);
    }

    pub fn pointer(&self) -> *const i32 {
        return self.lock_state.as_ptr();
    }
}

fn is_unlocked(state: i32) -> bool {
    return state == UNLOCKED;
}

fn has_writer(state: i32) -> bool {
    return state < 0;
}

fn wait(lock_state: &AtomicI32) {
    if !extern_functions::is_main_thread() {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            let ptr = lock_state.as_ptr();
            std::arch::wasm32::memory_atomic_wait32(ptr, 1, 1000_000);
        }
    }
}

fn notify(lock_state: &AtomicI32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        std::arch::wasm32::memory_atomic_notify(lock_state.as_ptr(), 999);
    }
}
