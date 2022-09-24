use std::{collections::HashMap, io::BufRead, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'f> {
    Num(i32),
    Op(String),
    Sym(String),
    Block(Vec<Value<'f>>),
    Native(NativeOp<'f>),
}

impl<'f> Value<'f> {
    fn as_num(&self) -> i32 {
        match self {
            Self::Num(val) => *val,
            _ => panic!("Value is not a number"),
        }
    }

    fn to_block(self) -> Vec<Value<'f>> {
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

impl<'f> ToString for Value<'f> {
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
pub struct NativeOp<'f>(Rc<Box<dyn Fn(&mut Vm) + 'f>>);

impl<'f> PartialEq for NativeOp<'f> {
    fn eq(&self, other: &NativeOp<'f>) -> bool {
        Rc::as_ptr(&self.0) == Rc::as_ptr(&other.0)
    }
}

impl<'f> Eq for NativeOp<'f> {}

impl<'f> std::fmt::Debug for NativeOp<'f> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<NativeOp>")
    }
}

#[derive(Debug)]
pub struct ExecFrame<'f> {
    block: Vec<Value<'f>>,
    ip: usize,
    pub vars: HashMap<String, Value<'f>>,
}

impl<'f> ExecFrame<'f> {
    fn new(block: Vec<Value<'f>>) -> Self {
        Self {
            block,
            ip: 0,
            vars: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum ExecState<'f> {
    Frame(ExecFrame<'f>),
    IfCond {
        frame: ExecFrame<'f>,
        true_branch: Vec<Value<'f>>,
        false_branch: Vec<Value<'f>>,
    },
    IfTrue(ExecFrame<'f>),
    IfFalse(ExecFrame<'f>),
}

impl<'f> ExecState<'f> {
    pub fn as_frame(&self) -> &ExecFrame<'f> {
        match self {
            Self::Frame(frame) => frame,
            Self::IfCond { frame, .. } => frame,
            Self::IfTrue(frame) | Self::IfFalse(frame) => frame,
        }
    }

    fn as_frame_mut(&mut self) -> &mut ExecFrame<'f> {
        match self {
            Self::Frame(frame) => frame,
            Self::IfCond { frame, .. } => frame,
            Self::IfTrue(frame) | Self::IfFalse(frame) => frame,
        }
    }
}

pub struct Vm<'f> {
    stack: Vec<Value<'f>>,
    globals: HashMap<String, Value<'f>>,
    exec_stack: Vec<ExecState<'f>>,
    blocks: Vec<Vec<Value<'f>>>,
}

impl<'f> Vm<'f> {
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
            globals: functions
                .into_iter()
                .map(|(name, fun)| {
                    (
                        name.to_owned(),
                        Value::Native(NativeOp(Rc::new(Box::new(fun)))),
                    )
                })
                .collect(),
            exec_stack: vec![],
            blocks: vec![vec![]],
        }
    }

    pub fn get_stack(&self) -> &[Value<'f>] {
        &self.stack
    }

    pub fn get_exec_stack(&self) -> &[ExecState<'f>] {
        &self.exec_stack
    }

    pub fn add_fn(&mut self, name: String, f: Box<dyn Fn(&mut Vm) + 'f>) {
        self.globals
            .insert(name, Value::Native(NativeOp(Rc::new(f))));
    }

    fn find_var(&self, name: &str) -> Option<Value<'f>> {
        self.exec_stack
            .iter()
            .rev()
            .find_map(|state| {
                if let ExecState::Frame(frame) = state {
                    frame.vars.get(name).cloned()
                } else {
                    None
                }
            })
            .or_else(|| self.globals.get(name).cloned())
    }

    pub fn get_vars(&self) -> &HashMap<String, Value> {
        &self.exec_stack.last().unwrap().as_frame().vars
    }

    pub fn parse_batch(&mut self, source: impl BufRead) {
        for line in source.lines().flatten() {
            for word in line.split(" ") {
                parse_word(word, self);
            }
        }

        if let Some(top_block) = self.blocks.first() {
            self.exec_stack
                .push(ExecState::Frame(ExecFrame::new(top_block.clone())));
        }
    }

    pub fn eval_all(&mut self) {
        while self.eval_step() {}
    }

    pub fn eval_step(&mut self) -> bool {
        let get_step = |frame: &mut ExecFrame<'f>| {
            if frame.ip < frame.block.len() {
                let code = frame.block[frame.ip].clone();
                frame.ip += 1;
                Some(code)
            } else {
                None
            }
        };

        if let Some(state) = self.exec_stack.last_mut() {
            match state {
                ExecState::Frame(frame)
                | ExecState::IfTrue(frame)
                | ExecState::IfFalse(frame) => {
                    if let Some(code) = get_step(frame) {
                        eval(&code, self);
                    } else {
                        self.exec_stack.pop();
                    }
                }
                ExecState::IfCond { frame, .. } => {
                    if let Some(code) = get_step(frame) {
                        eval(&code, self);
                    } else {
                        let cond = self.stack.pop().unwrap();
                        if cond.as_num() != 0 {
                            let block = if let ExecState::IfCond {
                                true_branch,
                                ..
                            } = self.exec_stack.pop().unwrap()
                            {
                                true_branch
                            } else {
                                panic!("Top should be IfCond!");
                            };
                            self.exec_stack
                                .push(ExecState::IfTrue(ExecFrame::new(block)));
                        } else {
                            let block = if let ExecState::IfCond {
                                false_branch,
                                ..
                            } = self.exec_stack.pop().unwrap()
                            {
                                false_branch
                            } else {
                                panic!("Top should be IfCond!");
                            };
                            self.exec_stack.push(ExecState::IfFalse(
                                ExecFrame::new(block),
                            ));
                        }
                    }
                }
            }
            true
        } else {
            false
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

fn parse_word(word: &str, vm: &mut Vm) {
    if word.is_empty() {
        return;
    }
    if word == "{" {
        vm.blocks.push(vec![]);
    } else if word == "}" {
        let new_block = vm.blocks.pop().expect("Block stack underflow!");
        if let Some(top_block) = vm.blocks.last_mut() {
            top_block.push(Value::Block(new_block));
        }
    } else if let Some(top_block) = vm.blocks.last_mut() {
        let code = if let Ok(num) = word.parse::<i32>() {
            Value::Num(num)
        } else if word.starts_with("/") {
            Value::Sym(word[1..].to_string())
        } else {
            Value::Op(word.to_string())
        };
        top_block.push(code);
        // eval(code, vm);
    }
}

fn eval<'f>(code: &Value<'f>, vm: &mut Vm<'f>) {
    if let Value::Op(ref op) = code {
        let val = vm
            .find_var(op)
            .expect(&format!("{op:?} is not a defined operation"));
        match val {
            Value::Block(block) => {
                vm.exec_stack.push(ExecState::Frame(ExecFrame::new(block)));
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

    vm.exec_stack.push(ExecState::IfCond {
        frame: ExecFrame::new(cond),
        true_branch,
        false_branch,
    });
}

fn op_def(vm: &mut Vm) {
    let value = vm.stack.pop().unwrap();
    eval(&value, vm);
    let value = vm.stack.pop().unwrap();
    let sym = vm.stack.pop().unwrap().as_sym().to_string();

    vm.exec_stack
        .last_mut()
        .unwrap()
        .as_frame_mut()
        .vars
        .insert(sym, value);
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
