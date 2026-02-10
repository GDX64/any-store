use std::{mem, sync::LazyLock};

use crate::{
    my_rwlock::MyRwLock,
    storage::{Database, Table},
    value::Something,
};

enum Operation {
    InsertRow {
        table_id: usize,
        data: Vec<Something>,
    },
    Insert {
        table_id: usize,
        key: Something,
        value: Something,
        index: usize,
    },
    RowDelete {
        table_id: usize,
        key: Something,
    },
}

static SOMETHING_STACK: LazyLock<MyRwLock<[Vec<Something>; 16]>> =
    LazyLock::new(|| MyRwLock::new(Default::default()));
static OPERATION_STACK: LazyLock<MyRwLock<[Vec<Operation>; 16]>> =
    LazyLock::new(|| MyRwLock::new(Default::default()));

fn push_to_something_stack(value: Something) {
    let mut stack = SOMETHING_STACK.write();
    let worker_id = worker_id();
    stack[worker_id].push(value);
}

fn pop_from_something_stack() -> Option<Something> {
    let mut stack = SOMETHING_STACK.write();
    let worker_id = worker_id();
    return stack[worker_id].pop();
}

struct GlobalPool {
    db: MyRwLock<Database>,
}

impl GlobalPool {
    fn new() -> Self {
        GlobalPool {
            db: MyRwLock::new(Database::new()),
        }
    }

    fn add_table(&self) -> usize {
        let mut db = self.db.write();
        return db.create_table();
    }

    fn with_db_mut<R, F: FnOnce(&mut Database) -> R>(&self, f: F) -> Option<R> {
        let mut pool = self.db.write();
        return Some(f(&mut pool));
    }

    // fn with_db<R, F: FnOnce(&Database) -> R>(&self, f: F) -> Option<R> {
    //     let pool = self.db.read();
    //     return Some(f(&pool));
    // }

    fn with_table<R, F: FnOnce(&Table) -> R>(&self, idx: usize, f: F) -> Option<R> {
        let pool = self.db.read();
        let value = pool.get_table(idx)?;
        return Some(f(&value));
    }
}

static GLOBALS: LazyLock<GlobalPool> = LazyLock::new(|| GlobalPool::new());

#[unsafe(no_mangle)]
pub fn start() {
    log_string(&format!("mod start with worker_id {}", worker_id()));
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        let full_message = format!("Panic occurred: {}", msg);
        log_string(&full_message);
    }));
}

#[unsafe(no_mangle)]
pub fn table_create() -> usize {
    return GLOBALS.add_table() as usize;
}

#[unsafe(no_mangle)]
pub fn table_get_something(table: usize, col: usize) -> Option<()> {
    let key = pop_from_something_stack()?;
    let something = GLOBALS.with_table(table, |table: &Table| {
        return table.get(&key).and_then(|row| {
            return Some(row.get(col).clone());
        });
    })??;
    add_something_to_js_stack(&something);
    return Some(());
}

#[unsafe(no_mangle)]
pub fn table_get_row(table: usize) -> Option<()> {
    let key = pop_from_something_stack()?;
    let row = GLOBALS.with_table(table, |table: &Table| {
        return table.get(&key).cloned();
    })??;
    for item in row.iter() {
        add_something_to_js_stack(&item);
    }
    return Some(());
}

#[unsafe(no_mangle)]
fn table_insert(table: usize, col: usize) -> Option<()> {
    let value = pop_from_something_stack()?;
    let key = pop_from_something_stack()?;
    let mut stack = OPERATION_STACK.write();
    stack[worker_id()].push(Operation::Insert {
        table_id: table,
        key,
        value,
        index: col,
    });
    return Some(());
}

#[unsafe(no_mangle)]
fn commit_ops() {
    GLOBALS.with_db_mut(|db| {
        let mut val = OPERATION_STACK.write();
        let ops = std::mem::take(&mut val[worker_id()]);
        for op in ops {
            match op {
                Operation::InsertRow { table_id, data } => {
                    db.get_table_mut(table_id).and_then(|table| {
                        return table.insert_row(data);
                    });
                }
                Operation::Insert {
                    table_id,
                    key,
                    value,
                    index,
                } => {
                    db.get_table_mut(table_id).map(|table| {
                        return table.insert_at(key, value, index);
                    });
                }
                Operation::RowDelete { table_id, key } => {
                    db.get_table_mut(table_id).map(|table| {
                        table.delete_row(&key);
                    });
                }
            }
        }
    });
}

#[unsafe(no_mangle)]
fn table_insert_row(table: usize) -> Option<()> {
    let v = {
        let mut val = SOMETHING_STACK.write();
        mem::take(&mut val[worker_id()])
    };
    let mut stack = OPERATION_STACK.write();
    stack[worker_id()].push(Operation::InsertRow {
        table_id: table,
        data: v,
    });
    return Some(());
}

#[unsafe(no_mangle)]
pub fn something_push_i32_to_stack(value: i32) {
    let something = Something::Int(value);
    push_to_something_stack(something);
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
    let something = Something::String(bytes);
    push_to_something_stack(something);
    return Some(());
}

#[unsafe(no_mangle)]
fn something_push_f64_to_stack(value: f64) {
    let something = Something::Float(value);
    push_to_something_stack(something);
}

#[unsafe(no_mangle)]
fn delete_row_from_table(table_id: usize) -> Option<()> {
    let something = pop_from_something_stack()?;
    let mut stack = OPERATION_STACK.write();
    stack[worker_id()].push(Operation::RowDelete {
        table_id,
        key: something,
    });
    return Some(());
}

#[unsafe(no_mangle)]
fn something_push_blob() -> Option<()> {
    let len = safe_read_blob_length();
    let mut bytes = Vec::with_capacity(len);
    for i in 0..len {
        let byte = safe_read_blob_byte(i);
        bytes.push(byte);
    }
    safe_js_pop_stack();
    let something = Something::Blob(bytes);
    push_to_something_stack(something);
    return Some(());
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
    unsafe fn js_log_stack_value();
    unsafe fn js_push_null();
    unsafe fn js_create_blob(size: usize);
    unsafe fn js_push_to_blob(byte: u8);
    unsafe fn js_read_blob_length() -> usize;
    unsafe fn js_read_blob_byte(index: usize) -> u8;
}

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    #[link_name = "worker_id"]
    fn unsafe_worker_id() -> i32;
}

fn worker_id() -> usize {
    unsafe {
        return unsafe_worker_id() as usize;
    }
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

fn safe_push_null() {
    unsafe {
        js_push_null();
    }
}

fn safe_log_stack_value() {
    unsafe {
        js_log_stack_value();
    }
}

fn log_string(message: &str) {
    safe_create_string();
    for byte in message.as_bytes() {
        safe_push_to_string(*byte);
    }
    safe_log_stack_value();
}

fn safe_create_blob(size: usize) {
    unsafe {
        js_create_blob(size);
    }
}

fn safe_push_to_blob(byte: u8) {
    unsafe {
        js_push_to_blob(byte);
    }
}

fn safe_read_blob_length() -> usize {
    unsafe {
        return js_read_blob_length();
    }
}

fn safe_read_blob_byte(index: usize) -> u8 {
    unsafe {
        return js_read_blob_byte(index);
    }
}
