use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    extern_functions::*,
    my_rwlock::{Lock, MyRwLock},
    storage::{Database, ListenerID, Operation},
    value::Something,
};
use std::sync::LazyLock;

thread_local! {
    static SOMETHING_STACK: std::cell::RefCell<Vec<Something>> = std::cell::RefCell::new(Vec::new());
}

fn pop_something() -> Option<Something> {
    return SOMETHING_STACK.with(|stack| stack.borrow_mut().pop());
}

fn push_something(value: Something) {
    SOMETHING_STACK.with(|stack| stack.borrow_mut().push(value));
}

fn take_something_stack() -> Vec<Something> {
    return SOMETHING_STACK.with(|stack| {
        let v = stack.clone();
        stack.borrow_mut().clear();
        return v.into_inner();
    });
}

fn push_to_something_stack(value: Something) {
    push_something(value);
}

fn pop_from_something_stack() -> Option<Something> {
    return pop_something();
}

static GLOBAL_LOCK: Lock = Lock::new();

struct GlobalPool {
    db: MyRwLock<Database>,
}

impl GlobalPool {
    fn new() -> Self {
        GlobalPool {
            db: MyRwLock::new(Database::new()),
        }
    }

    fn with_db_mut<R, F: FnOnce(&mut Database) -> R>(&self, f: F) -> Option<R> {
        let mut pool = self.db.write();
        return Some(f(&mut pool));
    }
}

static GLOBALS: LazyLock<GlobalPool> = LazyLock::new(|| GlobalPool::new());

#[wasm_bindgen]
pub fn lock() {
    GLOBAL_LOCK.lock();
}

#[wasm_bindgen]
pub fn unlock() {
    GLOBAL_LOCK.unlock();
}

#[wasm_bindgen]
pub fn try_lock() -> bool {
    return GLOBAL_LOCK.try_lock();
}

#[wasm_bindgen]
pub fn lock_pointer() -> *const i32 {
    return GLOBAL_LOCK.pointer();
}

#[wasm_bindgen]
pub fn start() {
    if worker_id() == 0 {
        std::panic::set_hook(Box::new(|info| {
            let msg = info.to_string();
            let full_message = format!("Panic occurred: {}", msg);
            log_string(&full_message);
        }));
    }
}

#[wasm_bindgen]
pub fn table_create() -> usize {
    return GLOBALS
        .with_db_mut(|db| {
            let name = pop_from_something_stack().expect("there should be a name for the table");
            return db.create_table(name);
        })
        .unwrap_or(0);
}

#[wasm_bindgen]
pub fn table_get_id_from_name() -> i32 {
    let name = pop_from_something_stack().expect("there should be a name for the table");
    return GLOBALS
        .with_db_mut(|db| {
            return db.get_table_id(name);
        })
        .unwrap_or(None)
        .map(|id| id as i32)
        .unwrap_or(-1);
}

#[wasm_bindgen]
pub fn table_get_something(table: usize, col: usize) {
    _table_get_something(table, col);
}

fn _table_get_something(table: usize, col: usize) -> Option<()> {
    let key = pop_from_something_stack()?;
    let something = GLOBALS.with_db_mut(|db| {
        let table = db.get_table(table)?;
        return table.get(&key).and_then(|row| {
            return Some(row.get(col).clone());
        });
    })??;
    add_something_to_js_stack(&something);
    return Some(());
}

#[wasm_bindgen]
pub fn table_get_row(table: usize) {
    _table_get_row(table);
}

fn _table_get_row(table: usize) -> Option<()> {
    let key = pop_from_something_stack()?;
    let row = GLOBALS.with_db_mut(|db| {
        let table = db.get_table(table)?;
        return table.get(&key).cloned();
    })??;
    for item in row.iter() {
        add_something_to_js_stack(&item);
    }
    return Some(());
}

#[wasm_bindgen]
pub fn table_insert(table: usize, col: usize) {
    let Some(value) = pop_from_something_stack() else {
        return;
    };
    let Some(key) = pop_from_something_stack() else {
        return;
    };
    GLOBALS.with_db_mut(|db| {
        db.operation(Operation::Insert {
            table_id: table,
            key,
            value,
            index: col,
        });
    });
}

#[wasm_bindgen]
pub fn table_insert_row(table: usize) {
    GLOBALS.with_db_mut(|db| {
        let v = take_something_stack();
        db.operation(Operation::InsertRow {
            table_id: table,
            data: v,
        });
    });
}

#[wasm_bindgen]
pub fn something_push_i32_to_stack(value: i32) {
    let something = Something::Int(value);
    push_to_something_stack(something);
}

#[wasm_bindgen]
pub fn something_push_string() {
    let len = safe_read_string_length();
    let mut bytes = Vec::with_capacity(len);
    for i in 0..len {
        let byte = safe_read_string(i);
        bytes.push(byte);
    }
    safe_js_pop_stack();
    let something = Something::String(bytes);
    push_to_something_stack(something);
}

#[wasm_bindgen]
pub fn something_push_f64_to_stack(value: f64) {
    let something = Something::Float(value);
    push_to_something_stack(something);
}

#[wasm_bindgen]
pub fn delete_row_from_table(table_id: usize) {
    let Some(something) = pop_from_something_stack() else {
        return;
    };
    GLOBALS.with_db_mut(|db| {
        db.operation(Operation::RowDelete {
            table_id,
            key: something,
        });
    });
}

#[wasm_bindgen]
pub fn table_remove_listener(table_id: usize, listener_id: u32) {
    let Some(key) = pop_from_something_stack() else {
        return;
    };
    GLOBALS.with_db_mut(|db| {
        db.remove_listener(table_id, &key, listener_id);
    });
}

#[wasm_bindgen]
pub fn db_take_notifications() {
    let Some(notifications) = GLOBALS.with_db_mut(|db| {
        return db.take_notifications(worker_id() as u8);
    }) else {
        return;
    };

    for notification in notifications {
        safe_put_i32(notification);
    }
}

#[wasm_bindgen]
pub fn table_add_listener_to_row(table_id: usize) -> i32 {
    fn inner(table_id: usize) -> Option<ListenerID> {
        let something = pop_from_something_stack()?;
        let id = GLOBALS.with_db_mut(|db| {
            return db.add_listener_to(table_id, &something);
        })?;
        return id;
    }
    return inner(table_id).map(|id| id.to_i32()).unwrap_or(-1);
}

#[wasm_bindgen]
pub fn something_push_blob() {
    let len = safe_read_blob_length();
    let mut bytes = Vec::with_capacity(len);
    for i in 0..len {
        let byte = safe_read_blob_byte(i);
        bytes.push(byte);
    }
    safe_js_pop_stack();
    let something = Something::Blob(bytes);
    push_to_something_stack(something);
}

fn add_something_to_js_stack(value: &Something) {
    match value {
        Something::Int(v) => {
            safe_put_i32(*v);
        }
        Something::String(s) => {
            safe_create_string();
            for byte in s {
                safe_push_to_string(*byte);
            }
        }
        Something::Blob(b) => {
            safe_create_blob(b.len());
            for byte in b {
                safe_push_to_blob(*byte);
            }
        }
        Something::Null => {
            safe_push_null();
        }
        Something::Float(f) => {
            safe_put_f64(*f);
        }
    }
}
