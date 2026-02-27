pub use extern_functions_mod::*;

#[derive(Debug, Clone, PartialEq)]
pub enum MockValue {
    Int(i32),
    Float(f64),
    String(Vec<u8>),
    Blob(Vec<u8>),
    Null,
}

#[cfg(target_arch = "wasm32")]
mod extern_functions_mod {
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::extern_functions::MockValue;

    #[wasm_bindgen]
    unsafe extern "C" {
        // unsafe fn log_message(ptr: *const u8, len: usize);

        #[wasm_bindgen]
        fn js_read_string(index: usize) -> u8;
        #[wasm_bindgen]
        fn js_push_to_string(byte: u8);
        #[wasm_bindgen]
        fn js_read_string_length() -> usize;
        #[wasm_bindgen]
        fn js_pop_stack();
        #[wasm_bindgen]
        fn js_push_string_to_stack();
        #[wasm_bindgen]
        fn js_put_i32(value: i32);
        #[wasm_bindgen]
        fn js_put_f64(value: f64);
        #[wasm_bindgen]
        fn js_log_stack_value();
        #[wasm_bindgen]
        fn js_push_null();
        #[wasm_bindgen]
        fn js_create_blob(size: usize);
        #[wasm_bindgen]
        fn js_push_to_blob(byte: u8);
        #[wasm_bindgen]
        fn js_read_blob_length() -> usize;
        #[wasm_bindgen]
        fn js_read_blob_byte(index: usize) -> u8;
        #[wasm_bindgen]
        fn unsafe_worker_id() -> i32;
    }

    pub fn is_main_thread() -> bool {
        worker_id() == 0
    }

    pub fn worker_id() -> usize {
        return unsafe_worker_id() as usize;
    }

    pub fn safe_read_string(index: usize) -> u8 {
        let byte = js_read_string(index);
        return byte;
    }

    pub fn safe_create_string() {
        js_push_string_to_stack();
    }

    pub fn safe_push_to_string(byte: u8) {
        js_push_to_string(byte);
    }

    pub fn safe_read_string_length() -> usize {
        let len = js_read_string_length();
        return len;
    }

    pub fn safe_put_i32(value: i32) {
        js_put_i32(value);
    }

    pub fn safe_put_f64(value: f64) {
        js_put_f64(value);
    }

    pub fn safe_js_pop_stack() {
        js_pop_stack();
    }

    pub fn safe_push_null() {
        js_push_null();
    }

    pub fn safe_log_stack_value() {
        js_log_stack_value();
    }

    pub fn log_string(message: &str) {
        safe_create_string();
        for byte in message.as_bytes() {
            safe_push_to_string(*byte);
        }
        safe_log_stack_value();
    }

    pub fn safe_create_blob(size: usize) {
        js_create_blob(size);
    }

    pub fn safe_push_to_blob(byte: u8) {
        js_push_to_blob(byte);
    }

    pub fn safe_read_blob_length() -> usize {
        return js_read_blob_length();
    }

    pub fn safe_read_blob_byte(index: usize) -> u8 {
        return js_read_blob_byte(index);
    }

    pub fn with_stack_mut<R>(_f: impl FnOnce(&mut Vec<MockValue>) -> R) -> R {
        panic!("Not implemented in wasm");
    }

    pub fn set_worker_id(_id: i32) {
        panic!("Not implemented in wasm");
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod extern_functions_mod {

    use std::cell::RefCell;

    use crate::extern_functions::MockValue;

    thread_local! {
        static MOCK_STRING_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
        static MOCK_BLOB_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
        static MOCK_STACK: RefCell<Vec<MockValue>> = RefCell::new(Vec::new());
        static MOCK_WORKER_ID: RefCell<i32> = RefCell::new(0);
        static MOCK_LOGS: RefCell<Vec<String>> = RefCell::new(Vec::new());
    }

    // Mock implementations
    pub fn safe_read_string(index: usize) -> u8 {
        MOCK_STRING_BUFFER.with(|buf| buf.borrow().get(index).copied().unwrap_or(0))
    }

    pub fn safe_push_to_string(byte: u8) {
        MOCK_STRING_BUFFER.with(|buf| buf.borrow_mut().push(byte));
    }

    pub fn safe_read_string_length() -> usize {
        MOCK_STRING_BUFFER.with(|buf| buf.borrow().len())
    }

    pub fn safe_js_pop_stack() {
        MOCK_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
    }

    pub fn safe_create_string() {
        MOCK_STRING_BUFFER.with(|buf| buf.borrow_mut().clear());
    }

    pub fn safe_put_i32(value: i32) {
        MOCK_STACK.with(|stack| stack.borrow_mut().push(MockValue::Int(value)));
    }

    pub fn safe_put_f64(value: f64) {
        MOCK_STACK.with(|stack| stack.borrow_mut().push(MockValue::Float(value)));
    }

    pub fn safe_log_stack_value() {
        MOCK_STACK.with(|stack| {
            if let Some(val) = stack.borrow().last() {
                MOCK_LOGS.with(|logs| {
                    logs.borrow_mut().push(format!("{:?}", val));
                });
            }
        });
    }

    pub fn safe_push_null() {
        MOCK_STACK.with(|stack| stack.borrow_mut().push(MockValue::Null));
    }

    pub fn safe_create_blob(size: usize) {
        MOCK_BLOB_BUFFER.with(|buf| {
            buf.borrow_mut().clear();
            buf.borrow_mut().reserve(size);
        });
    }

    pub fn safe_push_to_blob(byte: u8) {
        MOCK_BLOB_BUFFER.with(|buf| buf.borrow_mut().push(byte));
    }

    pub fn safe_read_blob_length() -> usize {
        MOCK_BLOB_BUFFER.with(|buf| buf.borrow().len())
    }

    pub fn safe_read_blob_byte(index: usize) -> u8 {
        MOCK_BLOB_BUFFER.with(|buf| buf.borrow().get(index).copied().unwrap_or(0))
    }

    pub fn worker_id() -> usize {
        MOCK_WORKER_ID.with(|id| *id.borrow() as usize)
    }

    pub fn log_string(s: &str) {
        MOCK_LOGS.with(|logs| logs.borrow_mut().push(s.to_string()));
    }

    pub fn is_main_thread() -> bool {
        worker_id() == 0
    }

    // Helper functions for tests
    pub fn set_worker_id(id: i32) {
        MOCK_WORKER_ID.with(|worker_id| *worker_id.borrow_mut() = id);
    }

    pub fn setup_mock_string(data: Vec<u8>) {
        MOCK_STRING_BUFFER.with(|buf| *buf.borrow_mut() = data.clone());
        MOCK_STACK.with(|stack| stack.borrow_mut().push(MockValue::String(data)));
    }

    pub fn setup_mock_blob(data: Vec<u8>) {
        MOCK_BLOB_BUFFER.with(|buf| *buf.borrow_mut() = data.clone());
        MOCK_STACK.with(|stack| stack.borrow_mut().push(MockValue::Blob(data)));
    }

    pub fn with_stack_mut<R>(f: impl FnOnce(&mut Vec<MockValue>) -> R) -> R {
        MOCK_STACK.with(|stack| f(&mut stack.borrow_mut()))
    }

    pub fn get_mock_logs() -> Vec<String> {
        MOCK_LOGS.with(|logs| logs.borrow().clone())
    }

    pub fn clear_all_mocks() {
        MOCK_STRING_BUFFER.with(|buf| buf.borrow_mut().clear());
        MOCK_BLOB_BUFFER.with(|buf| buf.borrow_mut().clear());
        MOCK_STACK.with(|stack| stack.borrow_mut().clear());
        MOCK_WORKER_ID.with(|id| *id.borrow_mut() = 0);
        MOCK_LOGS.with(|logs| logs.borrow_mut().clear());
    }
}
