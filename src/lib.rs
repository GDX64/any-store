pub mod storage;
mod tests;
pub mod value;
pub mod wasm;

use std::{
    any::Any,
    collections::BTreeMap,
    mem,
    sync::{LazyLock, Mutex},
};

use crate::{
    storage::Table,
    value::{ByteBuffer, Serializable, Something},
};

struct GlobalPool {
    pool: Mutex<BTreeMap<usize, Box<dyn Any>>>,
    stdin: Mutex<Vec<u8>>,
    something_stack: Mutex<Vec<Something>>,
}

impl GlobalPool {
    fn new() -> Self {
        GlobalPool {
            pool: Mutex::new(BTreeMap::new()),
            stdin: Mutex::new(Vec::new()),
            something_stack: Mutex::new(Vec::new()),
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
        let mut pool = self.pool.lock().unwrap();
        let value = pool
            .get_mut(&idx)?
            .downcast_mut::<T>()
            .expect("Type mismatch in with_box_value");
        return Some(f(value));
    }

    fn take_stdin(&self) -> ByteBuffer {
        let mut stdin = self.stdin.lock().unwrap();
        let data = mem::take(&mut *stdin);
        return ByteBuffer::from_vec(data);
    }

    fn read_stdin_something(&self) -> Something {
        let mut buffer = self.take_stdin();
        return Something::deserialize(&mut buffer);
    }

    fn push_to_something_stack(&self, value: Something) {
        let mut stack = self.something_stack.lock().unwrap();
        stack.push(value);
    }

    fn pop_from_something_stack(&self) -> Option<Something> {
        let mut stack = self.something_stack.lock().unwrap();
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
