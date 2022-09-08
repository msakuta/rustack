#[derive(Debug, Clone, PartialEq, Eq)]
enum Value<'src> {
    Num(i32),
    Op(&'src str),
    Block(Vec<Value<'src>>),
}

impl<'src> Value<'src> {
    fn as_num(&self) -> i32 {
        match self {
            Self::Num(val) => *val,
            _ => panic!("Value is not a number"),
        }
    }

    fn to_block(self) -> Vec<Value<'src>> {
        match self {
            Self::Block(val) => val,
            _ => panic!("Value is not a block"),
        }
    }
}

fn main() {
    for line in std::io::stdin().lines().flatten() {
        parse(&line);
    }
}

fn parse<'a>(line: &'a str) -> Vec<Value> {
    let mut stack = vec![];
    let input: Vec<_> = line.split(" ").collect();
    let mut words = &input[..];

    while let Some((&word, mut rest)) = words.split_first() {
        if word.is_empty() {
            break;
        }
        if word == "{" {
            let value;
            (value, rest) = parse_block(rest);
            stack.push(value);
        } else {
            let code = if let Ok(num) = word.parse::<i32>() {
                Value::Num(num)
            } else {
                Value::Op(word)
            };
            eval(code, &mut stack);
        }
        words = rest;
    }

    println!("stack: {stack:?}");

    stack
}

fn eval<'src>(code: Value<'src>, stack: &mut Vec<Value<'src>>) {
    match code {
        Value::Op(op) => match op {
            "+" => add(stack),
            "-" => sub(stack),
            "*" => mul(stack),
            "/" => div(stack),
            "if" => op_if(stack),
            _ => panic!("{op:?} could not be parsed"),
        },
        _ => stack.push(code.clone()),
    }
}

fn parse_block<'src, 'a>(input: &'a [&'src str]) -> (Value<'src>, &'a [&'src str]) {
    let mut tokens = vec![];
    let mut words = input;

    while let Some((&word, mut rest)) = words.split_first() {
        if word.is_empty() {
            break;
        }
        if word == "{" {
            let value;
            (value, rest) = parse_block(rest);
            tokens.push(value);
        } else if word == "}" {
            return (Value::Block(tokens), rest);
        } else if let Ok(value) = word.parse::<i32>() {
            tokens.push(Value::Num(value));
        } else {
            tokens.push(Value::Op(word));
        }
        words = rest;
    }

    (Value::Block(tokens), words)
}

fn add(stack: &mut Vec<Value>) {
    let rhs = stack.pop().unwrap().as_num();
    let lhs = stack.pop().unwrap().as_num();
    stack.push(Value::Num(lhs + rhs));
}

fn sub(stack: &mut Vec<Value>) {
    let rhs = stack.pop().unwrap().as_num();
    let lhs = stack.pop().unwrap().as_num();
    stack.push(Value::Num(lhs - rhs));
}

fn mul(stack: &mut Vec<Value>) {
    let rhs = stack.pop().unwrap().as_num();
    let lhs = stack.pop().unwrap().as_num();
    stack.push(Value::Num(lhs * rhs));
}

fn div(stack: &mut Vec<Value>) {
    let rhs = stack.pop().unwrap().as_num();
    let lhs = stack.pop().unwrap().as_num();
    stack.push(Value::Num(lhs / rhs));
}

fn op_if(stack: &mut Vec<Value>) {
    let false_branch = stack.pop().unwrap().to_block();
    let true_branch = stack.pop().unwrap().to_block();
    let cond = stack.pop().unwrap().to_block();

    for code in cond {
        eval(code, stack);
    }

    let cond_result = stack.pop().unwrap().as_num();

    if cond_result != 0 {
        for code in true_branch {
            eval(code, stack);
        }
    } else {
        for code in false_branch {
            eval(code, stack);
        }
    }
}

#[cfg(test)]
mod test {
    use super::{parse, Value::*};

    #[test]
    fn test_group() {
        assert_eq!(
            parse("1 2 + { 3 4 }"),
            vec![Num(3), Block(vec![Num(3), Num(4)])]
        );
    }

    #[test]
    fn test_if_true() {
        assert_eq!(parse("{ 1 -1 + } { 100 } { -100 } if"), vec![Num(-100)]);
    }

    #[test]
    fn test_if_false() {
        assert_eq!(parse("{ 1 1 + } { 100 } { -100 } if"), vec![Num(100)]);
    }
}
