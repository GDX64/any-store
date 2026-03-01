use std::thread;

use any_store::extern_functions::{MockValue, set_worker_id, with_stack_mut};
use any_store::js_things as js;

fn pop_mock_stack() -> Option<MockValue> {
    return with_stack_mut(|stack| stack.pop());
}

#[test]
fn multi_threaded() {
    with_stack_mut(|s| {
        s.push(MockValue::String("hello".into()));
    });
    js::something_push_string();
    let table = js::table_create();
    const COL: usize = 0;
    const N_REPETITIONS: usize = 10_000;
    js::something_push_i32_to_stack(0);
    let row_id = js::table_create_row(table) as u32;
    js::something_push_i32_to_stack(0);
    js::table_insert(table, COL, row_id);
    let test = move || {
        for _ in 0..N_REPETITIONS {
            js::lock();
            js::table_get_something(table, COL, row_id);
            let current_value = pop_mock_stack().unwrap();
            let MockValue::Int(current_value) = current_value else {
                panic!("expected int");
            };
            js::something_push_i32_to_stack(current_value + 1);
            js::table_insert(table, COL, row_id);
            js::unlock();
        }
    };
    let t1 = thread::spawn(move || {
        set_worker_id(1);
        test();
    });
    let t2 = thread::spawn(move || {
        set_worker_id(2);
        test();
    });

    t1.join().unwrap();
    t2.join().unwrap();

    js::something_push_i32_to_stack(0);
    js::table_get_something(table, COL, 0);
    let current_value = pop_mock_stack().unwrap();
    let MockValue::Int(current_value) = current_value else {
        panic!("expected int");
    };
    assert_eq!(current_value, (N_REPETITIONS as i32) * 2);
}
