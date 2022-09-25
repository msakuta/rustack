use rusty_stacker::Vm;
use wasm_bindgen::prelude::*;

pub(super) fn register_wasm_fn(vm: &mut Vm) {
    vm.add_fn("puts".to_string(), Box::new(puts));
    vm.add_fn("rectangle".to_string(), Box::new(rectangle));
    vm.add_fn("set_fill_style".to_string(), Box::new(set_fill_style));
    vm.add_fn("set_stroke_style".to_string(), Box::new(set_stroke_style));
    vm.add_fn("begin_path".to_string(), Box::new(begin_path));
    vm.add_fn("move_to".to_string(), Box::new(move_to));
    vm.add_fn("line_to".to_string(), Box::new(line_to));
    vm.add_fn("stroke".to_string(), Box::new(stroke));
    vm.add_fn("rotate".to_string(), Box::new(rotate));
    vm.add_fn("translate".to_string(), Box::new(translate));
    vm.add_fn("save".to_string(), Box::new(save));
    vm.add_fn("restore".to_string(), Box::new(restore));
}

#[wasm_bindgen(module = "/wasm_api.js")]
extern "C" {
    pub(crate) fn wasm_print(s: &str);
    pub(crate) fn wasm_rectangle(x0: f32, y0: f32, x1: f32, y1: f32);
    pub(crate) fn wasm_set_fill_style(s: &str);
    pub(crate) fn wasm_set_stroke_style(s: &str);
    pub(crate) fn wasm_begin_path();
    pub(crate) fn wasm_move_to(x0: f32, y0: f32);
    pub(crate) fn wasm_line_to(x0: f32, y0: f32);
    pub(crate) fn wasm_stroke();
    pub(crate) fn wasm_rotate(angle: f32);
    pub(crate) fn wasm_translate(x: f32, y: f32);
    pub(crate) fn wasm_save();
    pub(crate) fn wasm_restore();
}

fn puts(vm: &mut Vm) {
    wasm_print(&format!("puts: {}\n", vm.pop().unwrap().to_string()));
}

fn rectangle(vm: &mut Vm) {
    let y1 = vm.pop().unwrap().as_num();
    let x1 = vm.pop().unwrap().as_num();
    let y0 = vm.pop().unwrap().as_num();
    let x0 = vm.pop().unwrap().as_num();
    wasm_rectangle(x0, y0, x1, y1);
}

fn set_fill_style(vm: &mut Vm) {
    let b = vm.pop().unwrap().as_num();
    let g = vm.pop().unwrap().as_num();
    let r = vm.pop().unwrap().as_num();
    wasm_set_fill_style(&format!("rgb({r},{g},{b})"));
}

fn set_stroke_style(vm: &mut Vm) {
    let b = vm.pop().unwrap().as_num();
    let g = vm.pop().unwrap().as_num();
    let r = vm.pop().unwrap().as_num();
    wasm_set_stroke_style(&format!("rgb({r},{g},{b})"));
}

fn begin_path(_vm: &mut Vm) {
    wasm_begin_path();
}

fn move_to(vm: &mut Vm) {
    let y0 = vm.pop().unwrap().as_num();
    let x0 = vm.pop().unwrap().as_num();
    wasm_move_to(x0, y0);
}

fn line_to(vm: &mut Vm) {
    let y0 = vm.pop().unwrap().as_num();
    let x0 = vm.pop().unwrap().as_num();
    wasm_line_to(x0, y0);
}

fn stroke(_vm: &mut Vm) {
    wasm_stroke();
}

fn rotate(vm: &mut Vm) {
    let angle = vm.pop().unwrap().as_num();
    wasm_rotate(angle);
}

fn translate(vm: &mut Vm) {
    let y = vm.pop().unwrap().as_num();
    let x = vm.pop().unwrap().as_num();
    wasm_translate(x, y);
}

fn save(_vm: &mut Vm) {
    wasm_save();
}

fn restore(_vm: &mut Vm) {
    wasm_restore();
}
