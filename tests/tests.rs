use std::thread;

use any_store::extern_functions::{MockValue, pop_mock_stack, set_worker_id};
use any_store::js_things as js;

#[test]
fn it_works() {
    let table = js::table_create();
    const COL: usize = 0;
    js::something_push_i32_to_stack(17);
    js::something_push_i32_to_stack(28);
    js::table_insert(table, COL);
    js::commit_ops();
    js::something_push_i32_to_stack(17);
    js::table_get_something(table, COL).expect("must get something");
    let value = pop_mock_stack().unwrap();
    assert_eq!(value, MockValue::Int(28));
}

#[test]
fn multi_threaded() {
    let table = js::table_create();
    const COL: usize = 0;
    fn test(table: usize) {
        for _ in 0..10_000 {
            js::something_push_i32_to_stack(0);
            js::table_get_something(table, COL);
            let current_value = pop_mock_stack().unwrap_or(MockValue::Int(0));
            let MockValue::Int(current_value) = current_value else {
                panic!("expected int");
            };
            js::something_push_i32_to_stack(0);
            js::something_push_i32_to_stack(current_value + 1);
            js::table_insert(table, COL);
            js::commit_ops();
        }
    }
    let t1 = thread::spawn(move || {
        set_worker_id(1);
        test(table);
    });
    let t2 = thread::spawn(move || {
        set_worker_id(2);
        test(table);
    });

    t1.join().unwrap();
    t2.join().unwrap();

    js::something_push_i32_to_stack(0);
    js::table_get_something(table, COL);
    let current_value = pop_mock_stack().unwrap_or(MockValue::Int(0));
    let MockValue::Int(current_value) = current_value else {
        panic!("expected int");
    };
    assert!(current_value > 0);
}
