use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'vm> {
    Num(i32),
    Op(String),
    Sym(String),
    Block(Vec<Value<'vm>>),
    Native(NativeOp<'vm>),
}

impl<'vm> Value<'vm> {
    fn as_num(&self) -> i32 {
        match self {
            Self::Num(val) => *val,
            _ => panic!("Value is not a number"),
        }
    }

    fn to_block(self) -> Vec<Value<'vm>> {
        match self {
            Self::Block(val) => val,
            _ => panic!("Value is not a block"),
        }
    }

    fn as_sym(&self) -> &str {
        if let Self::Sym(sym) = self {
            sym
        } else {
            panic!("Value is not a symbol");
        }
    }
}

impl<'vm> ToString for Value<'vm> {
    fn to_string(&self) -> String {
        match self {
            Self::Num(i) => i.to_string(),
            Self::Op(ref s) | Self::Sym(ref s) => s.clone(),
            Self::Block(_) => "<Block>".to_string(),
            Self::Native(_) => "<Native>".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct NativeOp<'vm>(Rc<Box<dyn Fn(&mut Vm) + 'vm>>);

impl<'vm> PartialEq for NativeOp<'vm> {
    fn eq(&self, other: &NativeOp<'vm>) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl<'vm> Eq for NativeOp<'vm> {}

impl<'vm> std::fmt::Debug for NativeOp<'vm> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<NativeOp>")
    }
}

pub struct Vm<'vm> {
    stack: Vec<Value<'vm>>,
    vars: Vec<HashMap<String, Value<'vm>>>,
    blocks: Vec<Vec<Value<'vm>>>,
}

impl<'vm> Vm<'vm> {
    pub fn new() -> Self {
        let functions: [(&str, fn(&mut Vm)); 12] = [
            ("+", add),
            ("-", sub),
            ("*", mul),
            ("/", div),
            ("<", lt),
            ("if", op_if),
            ("def", op_def),
            ("puts", puts),
            ("pop", pop),
            ("dup", dup),
            ("exch", exch),
            ("index", index),
        ];
        Self {
            stack: vec![],
            vars: vec![functions
                .into_iter()
                .map(|(name, fun)| {
                    (
                        name.to_owned(),
                        Value::Native(NativeOp(Rc::new(Box::new(fun)))),
                    )
                })
                .collect()],
            blocks: vec![],
        }
    }

    pub fn get_stack(&self) -> &[Value<'vm>] {
        &self.stack
    }

    pub fn add_fn(&mut self, name: String, f: Box<dyn Fn(&mut Vm) + 'vm>) {
        self.vars
            .first_mut()
            .unwrap()
            .insert(name, Value::Native(NativeOp(Rc::new(f))));
    }

    fn find_var(&self, name: &str) -> Option<Value<'vm>> {
        self.vars
            .iter()
            .rev()
            .find_map(|vars| vars.get(name).map(|var| var.to_owned()))
    }

    pub fn parse_batch(&mut self, source: impl BufRead) {
        for line in source.lines().flatten() {
            for word in line.split(" ") {
                parse_word(word, self);
            }
        }
    }
}

pub fn parse_interactive() {
    let mut vm = Vm::new();
    for line in std::io::stdin().lines().flatten() {
        for word in line.split(" ") {
            parse_word(word, &mut vm);
        }
        println!("stack: {:?}", vm.stack);
    }
}

fn parse_word<'vm, 'f>(word: &str, vm: &'vm mut Vm<'f>) {
    if word.is_empty() {
        return;
    }
    if word == "{" {
        vm.blocks.push(vec![]);
    } else if word == "}" {
        let top_block = vm.blocks.pop().expect("Block stack underflow!");
        eval(Value::Block(top_block), vm);
    } else {
        let code = if let Ok(num) = word.parse::<i32>() {
            Value::Num(num)
        } else if word.starts_with("/") {
            Value::Sym(word[1..].to_string())
        } else {
            Value::Op(word.to_string())
        };
        eval(code, vm);
    }
}

fn eval<'f, 'vm>(code: Value<'f>, vm: &'vm mut Vm<'f>)
where
    'f: 'vm,
{
    if let Some(top_block) = vm.blocks.last_mut() {
        top_block.push(code);
        return;
    }
    if let Value::Op(ref op) = code {
        let val = vm
            .find_var(op)
            .expect(&format!("{op:?} is not a defined operation"));
        match val {
            Value::Block(block) => {
                vm.vars.push(HashMap::new());
                for code in block {
                    eval(code, vm);
                }
                vm.vars.pop();
            }
            Value::Native(op) => op.0(vm),
            _ => vm.stack.push(val),
        }
    } else {
        vm.stack.push(code.clone());
    }
}

macro_rules! impl_op {
    {$name:ident, $op:tt} => {
        fn $name(vm: &mut Vm) {
            let rhs = vm.stack.pop().unwrap().as_num();
            let lhs = vm.stack.pop().unwrap().as_num();
            vm.stack.push(Value::Num((lhs $op rhs) as i32));
        }
    }
}

impl_op!(add, +);
impl_op!(sub, -);
impl_op!(mul, *);
impl_op!(div, /);
impl_op!(lt, <);

fn op_if(vm: &mut Vm) {
    let false_branch = vm.stack.pop().unwrap().to_block();
    let true_branch = vm.stack.pop().unwrap().to_block();
    let cond = vm.stack.pop().unwrap().to_block();

    for code in cond {
        eval(code, vm);
    }

    let cond_result = vm.stack.pop().unwrap().as_num();

    if cond_result != 0 {
        for code in true_branch {
            eval(code, vm);
        }
    } else {
        for code in false_branch {
            eval(code, vm);
        }
    }
}

fn op_def(vm: &mut Vm) {
    let value = vm.stack.pop().unwrap();
    eval(value, vm);
    let value = vm.stack.pop().unwrap();
    let sym = vm.stack.pop().unwrap().as_sym().to_string();

    vm.vars.last_mut().unwrap().insert(sym, value);
}

fn puts(vm: &mut Vm) {
    let value = vm.stack.pop().unwrap();
    println!("{}", value.to_string());
}

fn pop(vm: &mut Vm) {
    vm.stack.pop().unwrap();
}

fn dup(vm: &mut Vm) {
    let value = vm.stack.last().unwrap();
    vm.stack.push(value.clone());
}

fn exch(vm: &mut Vm) {
    let last = vm.stack.pop().unwrap();
    let second = vm.stack.pop().unwrap();
    vm.stack.push(last);
    vm.stack.push(second);
}

fn index(vm: &mut Vm) {
    let index = vm.stack.pop().unwrap().as_num() as usize;
    let value = vm.stack[vm.stack.len() - index - 1].clone();
    vm.stack.push(value);
}

#[cfg(test)]
mod test {
    use super::{Value::*, *};
    use std::io::Cursor;

    fn parse(input: &str) -> Vec<Value> {
        let mut vm = Vm::new();
        vm.parse_batch(Cursor::new(input));
        vm.get_stack().to_vec()
    }

    #[test]
    fn test_group() {
        assert_eq!(
            parse("1 2 + { 3 4 }"),
            vec![Num(3), Block(vec![Num(3), Num(4)])]
        );
    }

    #[test]
    fn test_if_false() {
        assert_eq!(parse("{ 1 -1 + } { 100 } { -100 } if"), vec![Num(-100)]);
    }

    #[test]
    fn test_if_true() {
        assert_eq!(parse("{ 1 1 + } { 100 } { -100 } if"), vec![Num(100)]);
    }

    #[test]
    fn test_var() {
        assert_eq!(parse("/x 10 def /y 20 def x y *"), vec![Num(200)]);
    }

    #[test]
    fn test_var_if() {
        assert_eq!(
            parse("/x 10 def /y 20 def { x y < } { x } { y } if"),
            vec![Num(10)]
        );
    }

    #[test]
    fn test_multiline() {
        assert_eq!(
            parse(
                r#"
/x 10 def
/y 20 def

{ x y < }
{ x }
{ y }
if
"#
            ),
            vec![Num(10)]
        );
    }

    #[test]
    fn test_function() {
        assert_eq!(
            parse(
                r#"
/double { 2 * } def
10 double"#
            ),
            vec![Num(20)]
        );
    }
}
