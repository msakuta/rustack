mod utils;

use rusty_stacker::Vm;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub(crate) fn log(s: &str);
}

#[wasm_bindgen(module = "/wasm_api.js")]
extern "C" {
    pub(crate) fn wasm_print(s: &str);
    pub(crate) fn wasm_rectangle(x0: i32, y0: i32, x1: i32, y1: i32);
    pub(crate) fn wasm_set_fill_style(s: &str);
}

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
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
    tokens: Vec<String>,
}

#[wasm_bindgen]
pub fn start_step(src: String) -> VmHandle {
    let tokens = src
        .split([' ', '\t', '\r', '\n'])
        .filter_map(|tok| {
            if tok.is_empty() {
                None
            } else {
                Some(tok.to_owned())
            }
        })
        .collect();
    let mut vm = Vm::new();
    vm.parse_batch(std::io::Cursor::new(src));
    VmHandle { vm, tokens }
}

#[wasm_bindgen]
impl VmHandle {
    pub fn step(&mut self) -> Result<String, JsValue> {
        log(&format!("tokens: {:?}", self.tokens));
        if self.vm.eval_step() {
            let result = format!(
                "stack: {:?}\nvars: {:?}\nexec_stack: {:#?}",
                self.vm.get_stack(),
                self.vm.get_vars(),
                self.vm.get_exec_stack()
            );
            Ok(result)
        } else {
            return Err(JsValue::from_str("Input tokens exhausted"));
        }
    }
}
