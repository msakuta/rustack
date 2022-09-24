mod utils;

use rusty_stacker::Vm;
use serde::Serialize;
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
    let stack = {
        let mut vm = Vm::new();
        vm.add_fn("puts".to_string(), Box::new(puts));
        vm.parse_batch(std::io::Cursor::new(src));
        vm.eval_all();
        format!("stack: {:?}\n", vm.get_stack())
    };
    Ok(stack)
}

#[wasm_bindgen]
pub struct VmHandle {
    vm: Vm<'static>,
    tokens: Vec<String>,
}

fn puts(vm: &mut Vm) {
    wasm_print(&format!(
        "puts: {}\n",
        vm.get_stack().last().unwrap().to_string()
    ));
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
    vm.add_fn("puts".to_string(), Box::new(puts));
    vm.parse_batch(std::io::Cursor::new(src));
    VmHandle { vm, tokens }
}

#[wasm_bindgen]
impl VmHandle {
    pub fn step(&mut self) -> Result<(), JsValue> {
        log(&format!("tokens: {:?}", self.tokens));
        if self.vm.eval_step() {
            Ok(())
        } else {
            return Err(JsValue::from_str("Input tokens exhausted"));
        }
    }

    pub fn get_stack(&self) -> Result<Vec<JsValue>, JsValue> {
        Ok(self
            .vm
            .get_stack()
            .iter()
            .map(|val| JsValue::from_str(&format!("{:?}", val)))
            .collect())
    }

    /// Return execution stack in JSON string
    pub fn get_vars(&self) -> Result<String, JsValue> {
        #[derive(Serialize)]
        struct ExecFrame {
            name: String,
            /// We could return HashMap<String, String>, but it would be mapped to a JS object,
            /// which in turn changes order every time you run.
            vars: Vec<[String; 2]>,
        }

        let ret: Vec<ExecFrame> = self
            .vm
            .get_exec_stack()
            .iter()
            .map(|ex| {
                let frame = ex.as_frame();
                ExecFrame {
                    name: frame.name.clone(),
                    vars: frame
                        .vars
                        .iter()
                        .map(|(key, val)| [key.clone(), format!("{:?}", val)])
                        .collect(),
                }
            })
            .collect();
        let js = serde_json::to_string(&ret)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(js)
    }
}
