use std::{collections::HashMap, io::Read};

use crate::ast::FalseInstruction;

#[derive(Debug, Clone)]
enum FalseStoreableValue {
    StoredInteger(i32),
    StoredLambda(Vec<FalseInstruction>),
}

impl FalseStoreableValue {
    fn type_name(&self) -> &str {
        match self {
            Self::StoredInteger(_) => "Integer",
            Self::StoredLambda(_) => "Lambda",
        }
    }
}

#[derive(Debug, Clone)]
enum FalseStackEntry {
    VariableReference(char),
    StoredValue(FalseStoreableValue),
}
impl FalseStackEntry {
    fn type_name(&self) -> &str {
        match self {
            Self::VariableReference(_) => "VariableReference",
            Self::StoredValue(v) => v.type_name(),
        }
    }
}

struct FalseContext<'input_closure, 'output_closure> {
    stack: Vec<FalseStackEntry>,
    global_scope: HashMap<char, FalseStoreableValue>,
    on_input: Box<dyn 'input_closure + FnMut() -> Option<u8>>,
    on_output: Box<dyn 'output_closure + FnMut(String)>,
}

// Just a builder
pub struct Interpreter<'input_closure, 'output_closure> {
    on_input: Option<Box<dyn 'input_closure + FnMut() -> Option<u8>>>,
    on_output: Option<Box<dyn 'output_closure + FnMut(String)>>,
}
impl<'input_closure, 'output_closure> Interpreter<'input_closure, 'output_closure> {
    pub fn new() -> Self {
        Self {
            on_input: None,
            on_output: None,
        }
    }
    pub fn on_input<F: 'input_closure + FnMut() -> Option<u8>>(mut self, f: F) -> Self {
        self.on_input = Some(Box::new(f));
        self
    }
    pub fn on_output<F: 'output_closure + FnMut(String)>(mut self, f: F) -> Self {
        self.on_output = Some(Box::new(f));
        self
    }
    pub fn run_program(self, ast: Vec<FalseInstruction>) {
        let mut ctx = {
            FalseContext {
                on_input: self
                    .on_input
                    .unwrap_or_else(|| Box::new(default_read_input)),
                on_output: self.on_output.unwrap_or_else(|| Box::new(default_output)),
                stack: Vec::new(),
                global_scope: HashMap::new(),
            }
        };
        run_instructions(ast, &mut ctx);
    }
}

fn run_instructions(instructions: Vec<FalseInstruction>, ctx: &mut FalseContext) {
    for instruction in instructions {
        apply_instruction(instruction, ctx)
    }
}

fn apply_instruction(instruction: FalseInstruction, ctx: &mut FalseContext) {
    use FalseInstruction::*;
    use FalseStackEntry::*;
    use FalseStoreableValue::*;
    match instruction {
        Name(c) => ctx.stack.push(VariableReference(c)),
        PushInt(v) => ctx.stack.push(StoredValue(StoredInteger(v))),
        PushChar(c) => ctx.stack.push(StoredValue(StoredInteger(c.into()))),
        Dup => ctx.stack.push(ctx.stack.last().unwrap().clone()),
        Drop => {
            let _ = ctx.stack.pop();
        }
        Swap => {
            let head = ctx.stack.pop().unwrap();
            let next = ctx.stack.pop().unwrap();
            ctx.stack.extend_from_slice(&[head, next]);
        }
        Rot => {
            let first = ctx.stack.pop().unwrap();
            let second = ctx.stack.pop().unwrap();
            let third = ctx.stack.pop().unwrap();
            ctx.stack.extend_from_slice(&[first, second, third]);
        }
        Pick => {
            let head = ctx.stack.pop().unwrap();
            let index = match head {
                StoredValue(StoredInteger(i)) => i,
                other => panic!("Unexpected index for ø (PICK): {other:#?}"),
            };
            if index < 0 || index as usize >= ctx.stack.len() {
                panic!("Index out of rangte for ø (PICK): {index}");
            }
            let index = ctx.stack.len() - 1 - index as usize;
            ctx.stack.push(ctx.stack[index as usize].clone());
        }
        Add => binary_op(ctx, |a, b| a + b),
        Sub => binary_op(ctx, |a, b| a - b),
        Mul => binary_op(ctx, |a, b| a * b),
        Div => binary_op(ctx, |a, b| a / b),
        Neg => unary_op(ctx, |x| -x),
        BitAnd => binary_op(ctx, |a, b| a & b),
        BitOr => binary_op(ctx, |a, b| a | b),
        BitNot => unary_op(ctx, |x| !x),
        Gt => binary_op(ctx, |a, b| if a > b { -1 } else { 0 }),
        Eq => binary_op(ctx, |a, b| if a == b { -1 } else { 0 }),
        Lambda(vec) => ctx.stack.push(StoredValue(StoredLambda(vec))),
        Execute => {
            let lambda = match ctx.stack.pop().unwrap() {
                StoredValue(StoredLambda(v)) => v,
                other => panic!("Expected lambda for Execute, got {}", other.type_name()),
            };
            run_instructions(lambda, ctx);
        }
        ConditionalExecute(vec) => {
            let condition = pop_int(ctx);
            if condition != 0 {
                run_instructions(vec, ctx);
            }
        }
        WhileLoop(condition, body) => loop {
            run_instructions(condition.clone(), ctx);
            let condition_result = pop_int(ctx);
            if condition_result == 0 {
                break;
            }
            run_instructions(body.clone(), ctx);
        },
        Store => {
            let reference = match ctx.stack.pop().unwrap() {
                VariableReference(v) => v,
                _ => panic!("Store (:) must be preceded by a name"),
            };
            let value = match ctx.stack.pop().unwrap() {
                StoredValue(v) => v,
                VariableReference(_) => panic!("Names cannot be stored in names (I think?)"),
            };
            ctx.global_scope.insert(reference, value);
        }
        Fetch => {
            let reference = match ctx.stack.pop().unwrap() {
                VariableReference(v) => v,
                _ => panic!("Fetch (;) must be preceded by a name"),
            };
            let value = ctx.global_scope.get(&reference).unwrap().clone();
            ctx.stack.push(StoredValue(value));
        }
        ReadChar => {
            let value = match (*ctx.on_input)() {
                Some(v) => v as i32,
                None => -1,
            };
            ctx.stack.push(StoredValue(StoredInteger(value)));
        }
        WriteChar => {
            let value = pop_int(ctx);
            (*ctx.on_output)(std::char::from_u32(value as u32).unwrap().to_string());
        }
        WriteStr(s) => (*ctx.on_output)(s),
        WriteInt => {
            let value = pop_int(ctx);
            (*ctx.on_output)(value.to_string());
        }
        Flush => {}
    }
}

fn pop_two(ctx: &mut FalseContext) -> (i32, i32) {
    (pop_int(ctx), pop_int(ctx))
}

fn pop_int(ctx: &mut FalseContext) -> i32 {
    let head = ctx.stack.pop().unwrap();
    match head {
        FalseStackEntry::StoredValue(FalseStoreableValue::StoredInteger(i)) => i,
        other => panic!("Expected Integer on stack, got {}", other.type_name()),
    }
}

fn binary_op(ctx: &mut FalseContext, op: fn(i32, i32) -> i32) {
    let (a, b) = pop_two(ctx);
    ctx.stack.push(FalseStackEntry::StoredValue(
        FalseStoreableValue::StoredInteger(op(b, a)),
    ));
}

fn unary_op(ctx: &mut FalseContext, op: fn(i32) -> i32) {
    let a = pop_int(ctx);
    ctx.stack.push(FalseStackEntry::StoredValue(
        FalseStoreableValue::StoredInteger(op(a)),
    ));
}

fn default_read_input() -> Option<u8> {
    // Read 1 character from stdin
    eprint!("Input character: ");
    let mut buffer = [0; 1];
    std::io::stdin().read_exact(&mut buffer).ok()?;
    Some(buffer[0])
}

fn default_output(s: String) {
    print!("{s}");
}
