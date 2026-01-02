use std::{any::Any, cell::RefCell};

mod storage;
mod value;
mod wasm;

thread_local! {
    static GLOBALS: RefCell<Vec<Box<dyn Any>>> = RefCell::new(Vec::new());
}

#[unsafe(no_mangle)]
pub fn create_vec() -> usize {
    let v = vec![1, 2, 3];
    return put_in_any_box(v);
}

fn put_in_any_box<T: 'static>(value: T) -> usize {
    return GLOBALS.with(|globals| {
        let mut globals = globals.borrow_mut();
        let len = globals.len();
        globals.push(Box::new(value));
        return len;
    });
}

#[unsafe(no_mangle)]
pub fn push_vec(pointer: usize, value: i32) {
    with_box_value(pointer, |v: &mut Vec<i32>| {
        v.push(value);
    })
}

#[unsafe(no_mangle)]
pub fn get_vec(pointer: usize, index: usize) -> i32 {
    return with_box_value(pointer, |v: &mut Vec<i32>| {
        return v[index];
    });
}

fn with_box_value<T: 'static, R, F: FnOnce(&mut T) -> R>(idx: usize, f: F) -> R {
    return GLOBALS.with(|globals| {
        let mut globals = globals.borrow_mut();
        let value = globals[idx]
            .downcast_mut::<T>()
            .expect("Type mismatch in with_box_value");
        return f(value);
    });
}

#[cfg(test)]
mod tests {
    use crate::value::Something;

    use super::*;

    fn setup() -> storage::Table {
        let mut store = storage::Table::new(false);
        let v1 = Something::string("hello".into());
        let k1 = Something::Int(10);
        store.insert_at(k1.clone(), v1.clone(), 5);
        let v2 = Something::string("world".into());
        let k2 = Something::Int(20);
        store.insert_at(k2.clone(), v2.clone(), 3);
        let k3 = Something::Int(30);
        store.insert_at(k3.clone(), v1.clone(), 5);

        return store;
    }

    #[test]
    fn it_works() {
        let store = setup();
        let k1 = Something::Int(10);
        let value = store.get(&k1).map(|r| r.get(5));
        let v1 = Something::string("hello".into());
        assert_eq!(value, Some(&v1));
        let value = store.get(&Something::Int(-1));
        assert_eq!(value, None);
    }

    #[test]
    fn test_rows_with() {
        let store = setup();
        let rows = store.rows_with(|r| {
            return r.get(5) == &Something::string("hello".into());
        });

        assert_eq!(rows.count(), 2);
    }

    #[test]
    fn test_ordering() {
        let store = setup();
        let range = store.get_range(&Something::Int(15), &Something::Int(35));
        assert!(range.count() > 1);
    }

    #[test]
    fn test_replication() {
        let mut store = setup();
        let ops = store.take_ops();
        let store2 = storage::Table::from_ops(ops);
        let h1 = store.tree_hash();
        let h2 = store2.tree_hash();
        assert_eq!(h1, h2);
    }
}
