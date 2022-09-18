mod utils;

use wasm_bindgen::prelude::*;
use rusty_stacker::Vm;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm!");
}

#[wasm_bindgen]
pub fn entry(src: &str) -> Result<String, JsValue> {
    let mut buf = std::cell::RefCell::new("".to_string());
    let stack = {
        let mut vm = Vm::new();
        vm.add_fn("puts".to_string(), Box::new(|vm: &mut Vm| {
            *buf.borrow_mut() += &format!("puts: {}\n", vm.get_stack().last().unwrap().to_string()); })
        );
        vm.parse_batch(std::io::Cursor::new(src));
        format!("stack: {:?}\n", vm.get_stack())
    };
    let mut buf = buf.borrow().clone();
    buf += &stack;
    Ok(buf)
}