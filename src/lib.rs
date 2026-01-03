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
    string_pool: RwLock<BTreeMap<usize, String>>,
}

impl GlobalPool {
    fn new() -> Self {
        GlobalPool {
            pool: RwLock::new(BTreeMap::new()),
            something_stack: RwLock::new(Vec::new()),
            string_pool: RwLock::new(BTreeMap::new()),
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

    fn take_string(&self, str_idx: usize) -> Option<String> {
        let mut string_pool = self.string_pool.write().ok()?;
        let v = string_pool.remove(&str_idx);
        return v;
    }

    fn create_string(&self, value: String) -> usize {
        let mut string_pool = self.string_pool.write().unwrap();
        let new_key = if let Some((&key, _)) = string_pool.last_key_value() {
            key + 1
        } else {
            0
        };
        string_pool.insert(new_key, value);
        return new_key;
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
pub fn something_push_i64_to_stack(value: i64) {
    let something = Something::Int(value);
    GLOBALS.push_to_something_stack(something);
}

#[unsafe(no_mangle)]
pub fn something_pop_from_stack() -> i32 {
    let Some(value) = GLOBALS.pop_from_something_stack() else {
        return -1;
    };
    match value {
        Something::Int(v) => {
            let id = safe_next_id();
            safe_put_i64(id, v);
            return id as i32;
        }
        Something::String(s) => {
            let string_id = safe_create_string();
            for byte in s.as_bytes() {
                safe_push_to_string(string_id, *byte);
            }
            return string_id as i32;
        }
        Something::Null => {
            return -1;
        }
    }
}

#[unsafe(no_mangle)]
fn something_push_string(str_idx: usize) -> Option<()> {
    let s = GLOBALS.take_string(str_idx)?;
    let something = Something::String(s);
    GLOBALS.push_to_something_stack(something);
    return Some(());
}

#[unsafe(no_mangle)]
fn something_pop_string_from_stack() -> i32 {
    let something = GLOBALS.pop_from_something_stack();
    if let Some(Something::String(s)) = something {
        return GLOBALS.create_string(s) as i32;
    } else {
        return -1;
    }
}

#[unsafe(no_mangle)]
fn string_load(id: usize) -> usize {
    let len = safe_read_string_length(id);
    let mut bytes = Vec::with_capacity(len);
    for i in 0..len {
        let byte = safe_read_string(id, i);
        bytes.push(byte);
    }
    let s = String::from_utf8(bytes).unwrap();
    return GLOBALS.create_string(s);
}

#[unsafe(no_mangle)]
fn string_take(str_idx: usize) -> i32 {
    let s = GLOBALS.take_string(str_idx);
    if let Some(s) = s {
        let string_id = safe_create_string();
        for byte in s.as_bytes() {
            safe_push_to_string(string_id, *byte);
        }
        return string_id as i32;
    } else {
        return -1;
    }
}

#[link(wasm_import_module = "ops")]
unsafe extern "C" {
    // unsafe fn log_message(ptr: *const u8, len: usize);

    unsafe fn js_read_string(id: usize, index: usize) -> u8;
    unsafe fn js_push_to_string(string_id: usize, byte: u8);
    unsafe fn js_read_string_length(id: usize) -> usize;
    unsafe fn js_next_id() -> usize;
    unsafe fn js_put_i64(id: usize, value: i64);
}

fn safe_read_string(id: usize, index: usize) -> u8 {
    unsafe {
        let byte = js_read_string(id, index);
        return byte;
    }
}

fn safe_create_string() -> usize {
    unsafe {
        let id = js_next_id();
        return id;
    }
}

fn safe_push_to_string(string_id: usize, byte: u8) {
    unsafe {
        js_push_to_string(string_id, byte);
    }
}

fn safe_read_string_length(id: usize) -> usize {
    unsafe {
        let len = js_read_string_length(id);
        return len;
    }
}

fn safe_put_i64(id: usize, value: i64) {
    unsafe {
        js_put_i64(id, value);
    }
}

fn safe_next_id() -> usize {
    unsafe {
        let id = js_next_id();
        return id;
    }
}
