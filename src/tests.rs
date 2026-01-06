use crate::{storage, value::Something};

fn setup() -> storage::Table {
    let mut store = storage::Table::new();
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
