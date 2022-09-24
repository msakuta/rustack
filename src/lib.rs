use std::{collections::HashMap, io::BufRead, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'f> {
    Num(i32),
    Op(String),
    Sym(String),
    Block(BlockSpan<'f>),
    Native(NativeOp<'f>),
}

impl<'f> Value<'f> {
    fn as_num(&self) -> i32 {
        match self {
            Self::Num(val) => *val,
            _ => panic!("Value is not a number"),
        }
    }

    fn to_block(self) -> BlockSpan<'f> {
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
            Self::Block(block) => {
                format!("<Block [{},{}]>", block.span.0, block.span.1)
            }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueSpan<'f> {
    value: Value<'f>,
    span: (usize, usize),
}

#[derive(Debug)]
pub struct ExecFrame<'f> {
    pub name: String,
    block: BlockSpan<'f>,
    ip: usize,
    pub vars: HashMap<String, Value<'f>>,
}

impl<'f> ExecFrame<'f> {
    fn new(name: String, block: BlockSpan<'f>) -> Self {
        Self {
            name,
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
        true_branch: BlockSpan<'f>,
        false_branch: BlockSpan<'f>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSpan<'f> {
    block: Vec<ValueSpan<'f>>,
    span: (usize, usize),
}

impl<'f> BlockSpan<'f> {
    fn new(start: usize) -> Self {
        Self {
            block: vec![],
            span: (start, 0),
        }
    }
}

pub struct Vm<'f> {
    stack: Vec<Value<'f>>,
    globals: HashMap<String, Value<'f>>,
    exec_stack: Vec<ExecState<'f>>,
    blocks: Vec<BlockSpan<'f>>,
}

impl<'f> Vm<'f> {
    pub fn new() -> Self {
        let functions: [(&str, fn(&mut Vm)); 13] = [
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
            ("load", load),
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
            blocks: vec![BlockSpan::new(0)],
        }
    }

    pub fn get_stack(&self) -> &[Value<'f>] {
        &self.stack
    }

    pub fn pop(&mut self) -> Option<Value<'f>> {
        self.stack.pop()
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
        let mut tokenbuf = vec![];
        let mut byte_count = 0;
        for byte in source.bytes().map(|b| b.unwrap()) {
            match byte {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    parse_word(
                        std::str::from_utf8(&tokenbuf).unwrap(),
                        self,
                        byte_count - tokenbuf.len(),
                    );
                    tokenbuf.clear();
                }
                _ => tokenbuf.push(byte),
            }
            byte_count += 1;
        }

        parse_word(
            std::str::from_utf8(&tokenbuf).unwrap(),
            self,
            byte_count - tokenbuf.len(),
        );

        if let Some(top_block) = self.blocks.first() {
            self.exec_stack.push(ExecState::Frame(ExecFrame::new(
                "root".to_owned(),
                top_block.clone(),
            )));
        }
    }

    pub fn eval_all(&mut self) {
        while self.eval_step().is_some() {}
    }

    pub fn eval_step(&mut self) -> Option<(usize, usize)> {
        let get_step = |frame: &mut ExecFrame<'f>| {
            if frame.ip < frame.block.block.len() {
                let value_span = frame.block.block[frame.ip].clone();
                frame.ip += 1;
                Some(value_span)
            } else {
                None
            }
        };

        if let Some(state) = self.exec_stack.last_mut() {
            match state {
                ExecState::Frame(frame)
                | ExecState::IfTrue(frame)
                | ExecState::IfFalse(frame) => {
                    if let Some(value_span) = get_step(frame) {
                        eval(&value_span.value, self);
                        Some(value_span.span)
                    } else {
                        let frame = self.exec_stack.pop();
                        Some(
                            frame
                                .map(|frame| frame.as_frame().block.span)
                                .unwrap_or((0, 0)),
                        )
                    }
                }
                ExecState::IfCond { frame, .. } => {
                    if let Some(value_span) = get_step(frame) {
                        eval(&value_span.value, self);
                        Some(value_span.span)
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
                            let ret = block
                                .block
                                .first()
                                .map(|first| first.span)
                                .unwrap_or((0, 0));
                            self.exec_stack.push(ExecState::IfTrue(
                                ExecFrame::new("<IfTrue>".to_owned(), block),
                            ));
                            Some(ret)
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
                            let ret = block
                                .block
                                .first()
                                .map(|first| first.span)
                                .unwrap_or((0, 0));
                            self.exec_stack.push(ExecState::IfFalse(
                                ExecFrame::new("<IfFalse>".to_owned(), block),
                            ));
                            Some(ret)
                        }
                    }
                }
            }
        } else {
            None
        }
    }
}

pub fn parse_interactive() {
    let mut vm = Vm::new();
    for line in std::io::stdin().lines().flatten() {
        for word in line.split(" ") {
            let offset = word.as_ptr() as usize - line.as_ptr() as usize;
            parse_word(word, &mut vm, offset);
        }
        println!("stack: {:?}", vm.stack);
    }
}

fn parse_word(word: &str, vm: &mut Vm, offset: usize) {
    if word.is_empty() {
        return;
    }
    if word == "{" {
        vm.blocks.push(BlockSpan::new(offset));
    } else if word == "}" {
        let mut new_block = vm.blocks.pop().expect("Block stack underflow!");
        if let Some(top_block) = vm.blocks.last_mut() {
            new_block.span.1 = offset + 1;
            top_block.block.push(ValueSpan {
                span: (new_block.span.0, offset + 1),
                value: Value::Block(new_block),
            });
        }
    } else if let Some(top_block) = vm.blocks.last_mut() {
        let code = if let Ok(num) = word.parse::<i32>() {
            Value::Num(num)
        } else if word.starts_with("/") {
            Value::Sym(word[1..].to_string())
        } else {
            Value::Op(word.to_string())
        };
        top_block.block.push(ValueSpan {
            value: code,
            span: (offset, offset + word.len()),
        });
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
                vm.exec_stack
                    .push(ExecState::Frame(ExecFrame::new(op.clone(), block)));
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
        frame: ExecFrame::new("<IfCond>".to_owned(), cond),
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

fn load(vm: &mut Vm) {
    let key = vm.stack.pop().unwrap();
    let value = vm.find_var(key.as_sym()).unwrap();
    vm.stack.push(value);
}

#[cfg(test)]
mod test {
    use super::{Value::*, *};
    use std::io::Cursor;

    fn parse(input: &str) -> Vec<Value> {
        let mut vm = Vm::new();
        vm.parse_batch(Cursor::new(input));
        vm.eval_all();
        vm.get_stack().to_vec()
    }

    fn span(value: Value, span: (usize, usize)) -> ValueSpan {
        ValueSpan { value, span }
    }

    #[test]
    fn test_group() {
        assert_eq!(
            parse("1 2 + { 3 4 }"),
            vec![
                Num(3),
                Block(BlockSpan {
                    block: vec![span(Num(3), (8, 9)), span(Num(4), (10, 11))],
                    span: (6, 13),
                })
            ]
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
