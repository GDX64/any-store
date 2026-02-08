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
                Err(_) => {
                    let start = performance_now();
                    while performance_now() - start < 1.0 {
                        // spin
                    }
                }
            }
        }
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<T> {
        loop {
            match self.inner.try_write() {
                Ok(guard) => return guard,
                Err(_) => {
                    let start = performance_now();
                    while performance_now() - start < 1.0 {
                        // spin
                    }
                }
            }
        }
    }
}

#[link(wasm_import_module = "ops")]
unsafe extern "C" {
    unsafe fn js_performance_now() -> f64;
}

fn performance_now() -> f64 {
    unsafe {
        return js_performance_now();
    }
}
