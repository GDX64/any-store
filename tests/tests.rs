use any_store::extern_functions::{MockValue, with_stack_mut};
use any_store::js_things as js;

fn pop_mock_stack() -> Option<MockValue> {
    return with_stack_mut(|stack| stack.pop());
}

#[test]
fn it_works() {
    with_stack_mut(|s| {
        s.push(MockValue::String("hello".into()));
    });
    js::something_push_string();
    let table = js::table_create();
    const COL: usize = 0;
    js::something_push_i32_to_stack(17);
    js::something_push_i32_to_stack(28);
    js::table_insert(table, COL);
    js::something_push_i32_to_stack(17);
    js::table_get_something(table, COL).expect("must get something");
    let value = pop_mock_stack().unwrap();
    assert_eq!(value, MockValue::Int(28));
}
