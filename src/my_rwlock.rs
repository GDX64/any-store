use std::sync::RwLock;

pub struct MyRwLock<T> {
    inner: RwLock<T>,
}

impl<T> MyRwLock<T> {
    pub fn new(value: T) -> Self {
        MyRwLock {
            inner: RwLock::new(value),
        }
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<T> {
        loop {
            match self.inner.try_read() {
                Ok(guard) => return guard,
                Err(_) => {}
            }
        }
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<T> {
        loop {
            match self.inner.try_write() {
                Ok(guard) => return guard,
                Err(_) => {}
            }
        }
    }
}
