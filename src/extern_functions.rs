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
    use crate::extern_functions::MockValue;

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

    pub fn worker_id() -> usize {
        unsafe {
            return unsafe_worker_id() as usize;
        }
    }

    pub fn safe_read_string(index: usize) -> u8 {
        unsafe {
            let byte = js_read_string(index);
            return byte;
        }
    }

    pub fn safe_create_string() {
        unsafe {
            js_push_string_to_stack();
        }
    }

    pub fn safe_push_to_string(byte: u8) {
        unsafe {
            js_push_to_string(byte);
        }
    }

    pub fn safe_read_string_length() -> usize {
        unsafe {
            let len = js_read_string_length();
            return len;
        }
    }

    pub fn safe_put_i32(value: i32) {
        unsafe {
            js_put_i32(value);
        }
    }

    pub fn safe_put_f64(value: f64) {
        unsafe {
            js_put_f64(value);
        }
    }

    pub fn safe_js_pop_stack() {
        unsafe {
            js_pop_stack();
        }
    }

    pub fn safe_push_null() {
        unsafe {
            js_push_null();
        }
    }

    pub fn safe_log_stack_value() {
        unsafe {
            js_log_stack_value();
        }
    }

    pub fn log_string(message: &str) {
        safe_create_string();
        for byte in message.as_bytes() {
            safe_push_to_string(*byte);
        }
        safe_log_stack_value();
    }

    pub fn safe_create_blob(size: usize) {
        unsafe {
            js_create_blob(size);
        }
    }

    pub fn safe_push_to_blob(byte: u8) {
        unsafe {
            js_push_to_blob(byte);
        }
    }

    pub fn safe_read_blob_length() -> usize {
        unsafe {
            return js_read_blob_length();
        }
    }

    pub fn safe_read_blob_byte(index: usize) -> u8 {
        unsafe {
            return js_read_blob_byte(index);
        }
    }

    /**
     * this is just to fill the tests
     */
    pub fn pop_mock_stack() -> Option<MockValue> {
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

    pub fn pop_mock_stack() -> Option<MockValue> {
        MOCK_STACK.with(|stack| stack.borrow_mut().pop())
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
