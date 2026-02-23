use crate::{
    extern_functions::*,
    my_rwlock::{Lock, MyRwLock},
    storage::{Database, ListenerID, Operation},
    value::Something,
};
use std::{mem, sync::LazyLock};

fn push_to_something_stack(value: Something) {
    GLOBALS.with_db_mut(|db| {
        db.something_stack[worker_id()].push(value);
    });
}

fn pop_from_something_stack() -> Option<Something> {
    return GLOBALS.with_db_mut(|db| {
        return db.something_stack[worker_id()].pop();
    })?;
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

#[unsafe(no_mangle)]
pub fn lock() {
    GLOBAL_LOCK.lock();
}

#[unsafe(no_mangle)]
pub fn unlock() {
    GLOBAL_LOCK.unlock();
}

#[unsafe(no_mangle)]
pub fn start() {
    log_string(&format!("mod start with worker_id {}", worker_id()));
    if worker_id() == 0 {
        std::panic::set_hook(Box::new(|info| {
            let msg = info.to_string();
            let full_message = format!("Panic occurred: {}", msg);
            log_string(&full_message);
        }));
    }
}

#[unsafe(no_mangle)]
pub fn table_create() -> usize {
    return GLOBALS
        .with_db_mut(|db| {
            let name = pop_from_something_stack().expect("there shoud be a name for the table");
            return db.create_table(name);
        })
        .unwrap_or(0);
}

#[unsafe(no_mangle)]
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

#[unsafe(no_mangle)]
pub fn table_get_something(table: usize, col: usize) -> Option<()> {
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

#[unsafe(no_mangle)]
pub fn table_get_row(table: usize) -> Option<()> {
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

#[unsafe(no_mangle)]
pub fn table_insert(table: usize, col: usize) -> Option<()> {
    let value = pop_from_something_stack()?;
    let key = pop_from_something_stack()?;
    GLOBALS.with_db_mut(|db| {
        db.operation(Operation::Insert {
            table_id: table,
            key,
            value,
            index: col,
        });
    });
    return Some(());
}

#[unsafe(no_mangle)]
pub fn table_insert_row(table: usize) -> Option<()> {
    GLOBALS.with_db_mut(|db| {
        let v = { mem::take(&mut db.something_stack[worker_id()]) };
        db.operation(Operation::InsertRow {
            table_id: table,
            data: v,
        });
    });
    return Some(());
}

#[unsafe(no_mangle)]
pub fn something_push_i32_to_stack(value: i32) {
    let something = Something::Int(value);
    push_to_something_stack(something);
}

#[unsafe(no_mangle)]
pub fn something_push_string() -> Option<()> {
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
pub fn something_push_f64_to_stack(value: f64) {
    let something = Something::Float(value);
    push_to_something_stack(something);
}

#[unsafe(no_mangle)]
pub fn delete_row_from_table(table_id: usize) -> Option<()> {
    let something = pop_from_something_stack()?;
    GLOBALS.with_db_mut(|db| {
        db.operation(Operation::RowDelete {
            table_id,
            key: something,
        });
    });
    return Some(());
}

#[unsafe(no_mangle)]
pub fn table_remove_listener(table_id: usize, listener_id: u32) -> Option<()> {
    let key = pop_from_something_stack()?;
    GLOBALS.with_db_mut(|db| {
        db.remove_listener(table_id, &key, listener_id);
    });
    return Some(());
}

#[unsafe(no_mangle)]
pub fn db_take_notifications() -> Option<()> {
    let notifications = GLOBALS.with_db_mut(|db| {
        return db.take_notifications(worker_id() as u8);
    })?;

    for notification in notifications {
        safe_put_i32(notification);
    }

    return Some(());
}

#[unsafe(no_mangle)]
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

#[unsafe(no_mangle)]
pub fn something_push_blob() -> Option<()> {
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
