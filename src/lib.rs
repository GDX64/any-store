pub mod storage;
mod tests;
pub mod value;
pub mod wasm;

use std::{
    any::Any,
    collections::BTreeMap,
    sync::{LazyLock, Mutex},
};

use crate::{storage::Table, value::Something};

struct GlobalPool {
    pool: Mutex<BTreeMap<usize, Box<dyn Any>>>,
    something_stack: Mutex<Vec<Something>>,
    string_pool: Mutex<BTreeMap<usize, String>>,
}

impl GlobalPool {
    fn new() -> Self {
        GlobalPool {
            pool: Mutex::new(BTreeMap::new()),
            something_stack: Mutex::new(Vec::new()),
            string_pool: Mutex::new(BTreeMap::new()),
        }
    }

    fn put_in_any_box<T: 'static>(&self, value: T) -> usize {
        let mut pool = self.pool.lock().unwrap();
        let new_key = if let Some((&key, _)) = pool.last_key_value() {
            key + 1
        } else {
            0
        };
        pool.insert(new_key, Box::new(value));
        return new_key;
    }

    fn with_box_value<T: 'static, R, F: FnOnce(&mut T) -> R>(&self, idx: usize, f: F) -> Option<R> {
        let mut pool = self.pool.lock().ok()?;
        let value = pool.get_mut(&idx)?.downcast_mut::<T>()?;
        return Some(f(value));
    }

    fn get_string_pointer(&self, str_idx: usize) -> Option<*const u8> {
        let string_pool = self.string_pool.lock().unwrap();
        let v = string_pool.get(&str_idx).map(|s| s.as_ptr());
        return v;
    }

    fn get_string(&self, str_idx: usize) -> Option<String> {
        let mut string_pool = self.string_pool.lock().ok()?;
        let v = string_pool.remove(&str_idx);
        return v;
    }

    fn create_string(&self, value: String) -> usize {
        let mut string_pool = self.string_pool.lock().unwrap();
        let new_key = if let Some((&key, _)) = string_pool.last_key_value() {
            key + 1
        } else {
            0
        };
        string_pool.insert(new_key, value);
        return new_key;
    }

    fn push_to_something_stack(&self, value: Something) -> Option<()> {
        let mut stack = self.something_stack.lock().ok()?;
        stack.push(value);
        return Some(());
    }

    fn pop_from_something_stack(&self) -> Option<Something> {
        let mut stack = self.something_stack.lock().ok()?;
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
    let something = GLOBALS.with_box_value(table, |table: &mut Table| {
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
    return GLOBALS.with_box_value(table, |table: &mut storage::Table| {
        table.insert_at(key, value, col);
    });
}

#[unsafe(no_mangle)]
pub fn something_push_i64_to_stack(value: i64) {
    let something = Something::Int(value);
    GLOBALS.push_to_something_stack(something);
}

#[unsafe(no_mangle)]
pub fn something_pop_i64_from_stack() -> i64 {
    return _something_pop_i64_from_stack().unwrap_or(-1);
}

fn _something_pop_i64_from_stack() -> Option<i64> {
    let something = GLOBALS.pop_from_something_stack()?;
    if let Something::Int(v) = something {
        return Some(v);
    } else {
        return None;
    };
}

#[unsafe(no_mangle)]
fn string_create(len: usize) -> usize {
    let s = String::from_utf8(vec![0u8; len]).unwrap();
    return GLOBALS.create_string(s);
}

#[unsafe(no_mangle)]
fn string_get_pointer(str_idx: usize) -> *const u8 {
    let s = GLOBALS.get_string_pointer(str_idx).unwrap();
    return s;
}

#[unsafe(no_mangle)]
fn something_push_string(str_idx: usize) -> Option<()> {
    let s = GLOBALS.get_string(str_idx)?;
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
fn string_get_length(str_idx: usize) -> i32 {
    let s = GLOBALS.get_string(str_idx);
    if let Some(s) = s {
        let len = s.len();
        GLOBALS.create_string(s); // put it back
        return len as i32;
    } else {
        return -1;
    }
}
