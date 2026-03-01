use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    extern_functions::{self, *},
    my_rwlock::MyRwLock,
    storage::{Database, ListenerID, Operation},
    value::Something,
};
use std::{cell::RefCell, sync::LazyLock};

struct SomethingStack {
    stack: Vec<Something>,
}

impl SomethingStack {
    const fn new() -> Self {
        SomethingStack { stack: Vec::new() }
    }

    fn push(&mut self, value: Something) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<Something> {
        self.stack.pop()
    }
}

thread_local! {
    static SOMETHING_STACK: RefCell<SomethingStack> = RefCell::new(SomethingStack::new());
}

fn pop_something() -> Option<Something> {
    return SOMETHING_STACK.with(|stack| stack.borrow_mut().pop());
}

fn push_something(value: Something) {
    SOMETHING_STACK.with(|stack| stack.borrow_mut().push(value));
}

fn pop_from_something_stack() -> Option<Something> {
    return pop_something();
}

struct GlobalState {
    db: MyRwLock<Database>,
}

impl GlobalState {
    fn new() -> Self {
        GlobalState {
            db: MyRwLock::new(Database::new()),
        }
    }

    fn lock(&self) {
        self.db.lock.lock();
    }

    fn unlock(&self) {
        self.db.lock.unlock();
    }

    fn try_lock(&self) -> bool {
        return self.db.lock.try_lock();
    }

    fn lock_pointer(&self) -> *const i32 {
        return self.db.lock.pointer();
    }

    fn with_db_mut<R, F: FnOnce(&mut Database) -> R>(&self, f: F) -> Option<R> {
        let mut state = self.db.write();
        return Some(f(&mut state));
    }
}

static GLOBALS: LazyLock<GlobalState> = LazyLock::new(|| GlobalState::new());

#[wasm_bindgen]
pub fn lock() {
    GLOBALS.lock();
}

#[wasm_bindgen]
pub fn unlock() {
    GLOBALS.unlock();
}

#[wasm_bindgen]
pub fn try_lock() -> bool {
    return GLOBALS.try_lock();
}

#[wasm_bindgen]
pub fn lock_pointer() -> *const i32 {
    return GLOBALS.lock_pointer();
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
pub fn table_get_something(table: usize, col: usize, row_id: u32) {
    _table_get_something(table, col, row_id);
}

fn _table_get_something(table: usize, col: usize, row_id: u32) -> Option<()> {
    return GLOBALS.with_db_mut(|db| {
        let table = db.get_table(table)?;
        let row = table.get_row(row_id)?;
        push_to_js_stack(row.get(col));
        return Some(());
    })?;
}

#[wasm_bindgen]
pub fn table_get_row(table: usize, row_id: u32) {
    _table_get_row(table, row_id);
}

fn _table_get_row(table: usize, row_id: u32) -> Option<()> {
    return GLOBALS.with_db_mut(|db| {
        let table = db.get_table(table)?;
        let row = table.get_row(row_id)?;
        for item in row.iter() {
            push_to_js_stack(&item);
        }
        return Some(());
    })?;
}

#[wasm_bindgen]
pub fn table_insert(table: usize, col: usize, row_id: u32) {
    let Some(value) = pop_from_something_stack() else {
        return;
    };
    GLOBALS.with_db_mut(|db| {
        db.operation(Operation::Insert {
            table_id: table,
            row_id: row_id,
            value,
            index: col,
        });
    });
}

#[wasm_bindgen]
pub fn something_push_i32_to_stack(value: i32) {
    let something = Something::Int(value);
    push_something(something);
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
    push_something(something);
}

#[wasm_bindgen]
pub fn table_create_row(table: usize) -> i32 {
    let row_id = GLOBALS
        .with_db_mut(|db| {
            let key = pop_from_something_stack()?;
            let table = db.get_table_mut(table)?;
            return Some(table.create_row(key) as i32);
        })
        .unwrap_or(None)
        .unwrap_or(-1);
    return row_id;
}

#[wasm_bindgen]
pub fn table_with_col_equals(table: usize, col: usize) {
    GLOBALS.with_db_mut(|db| {
        let value = pop_from_something_stack()?;
        let table = db.get_table(table)?;
        let rows = table.with_cols_equal_to(col, value);
        for row_id in rows {
            push_to_js_stack(&Something::Int(row_id as i32));
        }
        return Some(());
    });
}

#[wasm_bindgen]
pub fn something_push_f64_to_stack(value: f64) {
    let something = Something::Float(value);
    push_something(something);
}

#[wasm_bindgen]
pub fn something_push_null_to_stack() {
    let something = Something::Null;
    push_something(something);
}

#[wasm_bindgen]
pub fn delete_row_from_table(table_id: usize, row_id: u32) {
    GLOBALS.with_db_mut(|db| {
        db.operation(Operation::RowDelete { table_id, row_id });
    });
}

#[wasm_bindgen]
pub fn table_remove_listener(table_id: usize, listener_id: u32, row_id: u32) {
    GLOBALS.with_db_mut(|db| {
        db.remove_listener(table_id, row_id, listener_id);
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
pub fn table_add_listener_to_row(table_id: usize, row_id: u32) -> i32 {
    fn inner(table_id: usize, row_id: u32) -> Option<ListenerID> {
        let id = GLOBALS.with_db_mut(|db| {
            return db.add_listener_to(table_id, row_id);
        })?;
        return id;
    }
    return inner(table_id, row_id).map(|id| id.to_i32()).unwrap_or(-1);
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
    push_something(something);
}

fn push_to_js_stack(value: &Something) {
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
