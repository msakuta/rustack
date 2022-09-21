mod utils;

use rusty_stacker::Vm;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/wasm_api.js")]
extern "C" {
    pub(crate) fn wasm_print(s: &str);
    pub(crate) fn wasm_rectangle(x0: i32, y0: i32, x1: i32, y1: i32);
    pub(crate) fn wasm_set_fill_style(s: &str);
}

#[wasm_bindgen]
pub fn entry(src: &str) -> Result<String, JsValue> {
    let mut buf = std::cell::RefCell::new("".to_string());
    let stack = {
        let mut vm = Vm::new();
        vm.add_fn(
            "puts".to_string(),
            Box::new(|vm: &mut Vm| {
                *buf.borrow_mut() += &format!(
                    "puts: {}\n",
                    vm.get_stack().last().unwrap().to_string()
                );
            }),
        );
        vm.parse_batch(std::io::Cursor::new(src));
        format!("stack: {:?}\n", vm.get_stack())
    };
    let mut buf = buf.borrow().clone();
    buf += &stack;
    Ok(buf)
}

#[wasm_bindgen]
pub struct VmHandle {
    vm: Vm<'static>,
    src: String,
}

#[wasm_bindgen]
pub fn start_step(src: String) -> VmHandle {
    VmHandle { vm: Vm::new(), src }
}

#[wasm_bindgen]
impl VmHandle {
    pub fn step(&mut self) -> String {
        self.vm.parse_step(std::io::Cursor::new(&self.src));
        format!("{:?}", self.vm.get_stack())
    }
}
