use std::{
    cell::RefCell,
    sync::{LazyLock, RwLock},
};

use crate::{
    storage::{self, Database, Table},
    value::Something,
};

thread_local! {
    static SOMETHING_STACK: RefCell<Vec<Something>> = RefCell::new(Vec::new());
}

fn push_to_something_stack(value: Something) {
    SOMETHING_STACK.with(|stack| {
        stack.borrow_mut().push(value);
    });
}

fn pop_from_something_stack() -> Option<Something> {
    SOMETHING_STACK.with(|stack| {
        return stack.borrow_mut().pop();
    })
}

struct GlobalPool {
    db: RwLock<Database>,
}

impl GlobalPool {
    fn new() -> Self {
        GlobalPool {
            db: RwLock::new(Database::new()),
        }
    }

    fn add_table(&self) -> usize {
        let mut db = self.db.write().unwrap();
        return db.create_table();
    }

    fn with_table_mut<R, F: FnOnce(&mut Table) -> R>(&self, idx: usize, f: F) -> Option<R> {
        let mut pool = self.db.write().ok()?;
        let value = pool.get_table_mut(idx)?;
        return Some(f(value));
    }

    fn with_table<R, F: FnOnce(&Table) -> R>(&self, idx: usize, f: F) -> Option<R> {
        let pool = self.db.read().ok()?;
        let value = pool.get_table(idx)?;
        return Some(f(value));
    }
}

unsafe impl Send for GlobalPool {}
unsafe impl Sync for GlobalPool {}

static GLOBALS: LazyLock<GlobalPool> = LazyLock::new(|| GlobalPool::new());

#[unsafe(no_mangle)]
pub fn start() {
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
    push_to_something_stack(something);
    return Some(());
}

#[unsafe(no_mangle)]
fn table_insert_from_stack(table: usize, col: usize) -> Option<()> {
    let value = pop_from_something_stack()?;
    let key = pop_from_something_stack()?;
    return GLOBALS.with_table_mut(table, |table: &mut storage::Table| {
        table.insert_at(key, value, col);
    });
}

#[unsafe(no_mangle)]
fn table_insert_row(table: usize) -> Option<()> {
    let v = SOMETHING_STACK.take();
    return GLOBALS.with_table_mut(table, |table: &mut storage::Table| {
        table.insert_row(v);
    });
}

#[unsafe(no_mangle)]
pub fn something_push_i32_to_stack(value: i32) {
    let something = Something::Int(value);
    push_to_something_stack(something);
}

#[unsafe(no_mangle)]
pub fn something_pop_from_stack() {
    let Some(value) = pop_from_something_stack() else {
        return;
    };
    add_something_to_js_stack(&value);
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
    push_to_something_stack(something);
    return Some(());
}

#[unsafe(no_mangle)]
fn something_push_f64_to_stack(value: f64) {
    let something = Something::Float(value);
    push_to_something_stack(something);
}

fn add_something_to_js_stack(value: &Something) {
    match value {
        Something::Int(v) => {
            safe_put_i32(*v);
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
