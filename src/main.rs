use std::error::Error;

use ::rustack::Vm;

pub fn main() -> Result<(), Box<dyn Error>> {
  let mut file_name = None;
  for arg in std::env::args().skip(1) {
    file_name = Some(arg);
  }
  let Some(file_name) = file_name else {
    eprintln!("usage: rustack [file_name.txt]");
    return Ok(());
  };
  let src = std::fs::read_to_string(file_name)?;
  let mut vm = Vm::new();
  vm.parse_batch(std::io::Cursor::new(src));
  vm.eval_all();
  format!("stack: {:?}\n", vm.get_stack());
  Ok(())
}
