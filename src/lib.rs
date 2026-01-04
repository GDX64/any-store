pub mod storage;
mod tests;
pub mod value;
pub mod wasm;

use std::{
    any::Any,
    collections::BTreeMap,
    sync::{LazyLock, RwLock},
};

use crate::{storage::Table, value::Something};

struct GlobalPool {
    pool: RwLock<BTreeMap<usize, Box<dyn Any>>>,
    something_stack: RwLock<Vec<Something>>,
}

impl GlobalPool {
    fn new() -> Self {
        GlobalPool {
            pool: RwLock::new(BTreeMap::new()),
            something_stack: RwLock::new(Vec::new()),
        }
    }

    fn put_in_any_box<T: 'static>(&self, value: T) -> usize {
        let mut pool = self.pool.write().unwrap();
        let new_key = if let Some((&key, _)) = pool.last_key_value() {
            key + 1
        } else {
            0
        };
        pool.insert(new_key, Box::new(value));
        return new_key;
    }

    fn with_box_value_mut<T: 'static, R, F: FnOnce(&mut T) -> R>(
        &self,
        idx: usize,
        f: F,
    ) -> Option<R> {
        let mut pool = self.pool.write().ok()?;
        let value = pool.get_mut(&idx)?.downcast_mut::<T>()?;
        return Some(f(value));
    }

    fn with_box_value<T: 'static, R, F: FnOnce(&T) -> R>(&self, idx: usize, f: F) -> Option<R> {
        let pool = self.pool.read().ok()?;
        let value = pool.get(&idx)?.downcast_ref::<T>()?;
        return Some(f(value));
    }

    fn push_to_something_stack(&self, value: Something) -> Option<()> {
        let mut stack = self.something_stack.write().ok()?;
        stack.push(value);
        return Some(());
    }

    fn pop_from_something_stack(&self) -> Option<Something> {
        let mut stack = self.something_stack.write().ok()?;
        return stack.pop();
    }
}

unsafe impl Send for GlobalPool {}
unsafe impl Sync for GlobalPool {}

static GLOBALS: LazyLock<GlobalPool> = LazyLock::new(|| GlobalPool::new());

#[unsafe(no_mangle)]
pub fn table_create() -> usize {
    let table = storage::Table::new();
    return GLOBALS.put_in_any_box(table);
}

#[unsafe(no_mangle)]
pub fn table_get_something(table: usize, col: usize) -> Option<()> {
    let key = GLOBALS.pop_from_something_stack()?;
    let something = GLOBALS.with_box_value(table, |table: &Table| {
        return table.get(&key).and_then(|row| {
            return Some(row.get(col).clone());
        });
    })??;
    GLOBALS.push_to_something_stack(something);
    return Some(());
}

#[unsafe(no_mangle)]
fn table_insert_from_stack(table: usize, col: usize) -> Option<()> {
    let value = GLOBALS.pop_from_something_stack()?;
    let key = GLOBALS.pop_from_something_stack()?;
    return GLOBALS.with_box_value_mut(table, |table: &mut storage::Table| {
        table.insert_at(key, value, col);
    });
}

#[unsafe(no_mangle)]
pub fn something_push_i32_to_stack(value: i32) {
    let something = Something::Int(value);
    GLOBALS.push_to_something_stack(something);
}

#[unsafe(no_mangle)]
pub fn something_pop_from_stack() {
    let Some(value) = GLOBALS.pop_from_something_stack() else {
        return;
    };
    match value {
        Something::Int(v) => {
            safe_put_i32(v);
        }
        Something::String(s) => {
            safe_create_string();
            for byte in s.as_bytes() {
                safe_push_to_string(*byte);
            }
        }
        Something::Null => {
            return;
        }
        Something::Float(f) => {
            safe_put_f64(f);
        }
    }
}

#[unsafe(no_mangle)]
fn something_push_string() -> Option<()> {
    let len = safe_read_string_length();
    let mut bytes = Vec::with_capacity(len);
    for i in 0..len {
        let byte = safe_read_string(i);
        bytes.push(byte);
    }
    safe_js_pop_stack();
    let s = String::from_utf8(bytes).unwrap();
    let something = Something::String(s);
    GLOBALS.push_to_something_stack(something);
    return Some(());
}

#[unsafe(no_mangle)]
fn something_push_f64_to_stack(value: f64) {
    let something = Something::Float(value);
    GLOBALS.push_to_something_stack(something);
}

#[link(wasm_import_module = "ops")]
unsafe extern "C" {
    // unsafe fn log_message(ptr: *const u8, len: usize);

    unsafe fn js_read_string(index: usize) -> u8;
    unsafe fn js_push_to_string(byte: u8);
    unsafe fn js_read_string_length() -> usize;
    unsafe fn js_pop_stack();
    unsafe fn js_push_string_to_stack();
    unsafe fn js_put_i32(value: i32);
    unsafe fn js_put_f64(value: f64);
}

fn safe_read_string(index: usize) -> u8 {
    unsafe {
        let byte = js_read_string(index);
        return byte;
    }
}

fn safe_create_string() {
    unsafe {
        js_push_string_to_stack();
    }
}

fn safe_push_to_string(byte: u8) {
    unsafe {
        js_push_to_string(byte);
    }
}

fn safe_read_string_length() -> usize {
    unsafe {
        let len = js_read_string_length();
        return len;
    }
}

fn safe_put_i32(value: i32) {
    unsafe {
        js_put_i32(value);
    }
}

fn safe_put_f64(value: f64) {
    unsafe {
        js_put_f64(value);
    }
}

fn safe_js_pop_stack() {
    unsafe {
        js_pop_stack();
    }
}
