use any_store::extern_functions::{MockValue, pop_mock_stack};
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
