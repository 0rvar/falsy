use std::{collections::HashMap, io::Read};

use chumsky::span::SimpleSpan;

use crate::ast::{FalseInstruction, Spanned};

pub struct InterpreterRuntimeError {
    span: SimpleSpan<usize>,
    reason: String,
}
impl std::fmt::Display for InterpreterRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}
impl std::fmt::Debug for InterpreterRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InterpreterRuntimeError {{ reason: {:?} }}", self.reason)
    }
}
impl InterpreterRuntimeError {
    pub fn new(span: SimpleSpan<usize>, reason: String) -> Self {
        Self { span, reason }
    }
    pub fn span(&self) -> SimpleSpan<usize> {
        self.span.clone()
    }
    pub fn reason(&self) -> &str {
        &self.reason
    }
}
impl std::error::Error for InterpreterRuntimeError {}

#[derive(Debug, Clone)]
enum FalseStoreableValue<'a> {
    StoredInteger(i32),
    StoredLambda(&'a [Spanned<FalseInstruction>]),
}

impl<'a> FalseStoreableValue<'a> {
    fn type_name(&self) -> &str {
        match self {
            Self::StoredInteger(_) => "Integer",
            Self::StoredLambda(_) => "Lambda",
        }
    }
}

#[derive(Debug, Clone)]
enum FalseStackEntry<'a> {
    VariableReference(char),
    StoredValue(FalseStoreableValue<'a>),
}
impl<'a> FalseStackEntry<'a> {
    fn type_name(&self) -> &str {
        match self {
            Self::VariableReference(_) => "VariableReference",
            Self::StoredValue(v) => v.type_name(),
        }
    }
}

struct FalseContext<'input_closure, 'output_closure, 'a> {
    stack: Vec<FalseStackEntry<'a>>,
    global_scope: HashMap<char, FalseStoreableValue<'a>>,
    on_input: Box<dyn 'input_closure + FnMut() -> Option<u8>>,
    on_output: Box<dyn 'output_closure + FnMut(&str)>,
}

// Just a builder
pub struct Interpreter<'input_closure, 'output_closure> {
    on_input: Option<Box<dyn 'input_closure + FnMut() -> Option<u8>>>,
    on_output: Option<Box<dyn 'output_closure + FnMut(&str)>>,
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
    pub fn on_output<F: 'output_closure + FnMut(&str)>(mut self, f: F) -> Self {
        self.on_output = Some(Box::new(f));
        self
    }
    pub fn run_program(
        self,
        ast: Vec<Spanned<FalseInstruction>>,
    ) -> Result<(), InterpreterRuntimeError> {
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
        run_instructions(&ast, &mut ctx)
    }
}

fn run_instructions<'a>(
    instructions: &'a [Spanned<FalseInstruction>],
    ctx: &mut FalseContext<'_, '_, 'a>,
) -> Result<(), InterpreterRuntimeError> {
    for instruction in instructions {
        apply_instruction(instruction, ctx)?
    }
    Ok(())
}

fn apply_instruction<'a>(
    spanned: &'a Spanned<FalseInstruction>,
    ctx: &mut FalseContext<'_, '_, 'a>,
) -> Result<(), InterpreterRuntimeError> {
    use FalseInstruction::*;
    use FalseStackEntry::*;
    use FalseStoreableValue::*;

    let span = spanned.span();

    macro_rules! runtime_error {
        ($reason:expr) => {
            InterpreterRuntimeError::new(span, format!($reason))
        };
        ($reason:expr, $($arg:expr),*) => {
            InterpreterRuntimeError::new(span, format!($reason, $($arg),*))
        };
    }

    macro_rules! error_factory {
        ($reason:expr) => {
            || runtime_error!($reason)
        };
        ($reason:expr, $($arg:expr),*) => {
            || runtime_error!($reason, $($arg),*)
        };
    }

    match spanned.instruction() {
        Name(c) => ctx.stack.push(VariableReference(*c)),
        PushInt(v) => ctx.stack.push(StoredValue(StoredInteger(*v))),
        PushChar(c) => ctx.stack.push(StoredValue(StoredInteger((*c).into()))),
        Dup => ctx.stack.push(
            ctx.stack
                .last()
                .ok_or_else(error_factory!("Stack is empty"))?
                .clone(),
        ),
        Drop => {
            let _ = ctx.stack.pop();
        }
        Swap => {
            let head = ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack is empty"))?;
            let next = ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack only has 1 value"))?;
            ctx.stack.extend_from_slice(&[head, next]);
        }
        Rot => {
            let first = ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack is empty"))?;
            let second = ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack only has 1 value"))?;
            let third = ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack only has 2 values"))?;
            ctx.stack.extend_from_slice(&[first, second, third]);
        }
        Pick => {
            let head = ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack is empty"))?;
            let index = match head {
                StoredValue(StoredInteger(i)) => i,
                other => return Err(runtime_error!("Unexpected index for ø (PICK): {other:#?}")),
            };
            if index < 0 || index as usize >= ctx.stack.len() {
                return Err(runtime_error!("Index out of range for ø (PICK): {index}"));
            }
            let index = ctx.stack.len() - 1 - index as usize;
            ctx.stack.push(ctx.stack[index as usize].clone());
        }
        Add => binary_op(ctx, span, |a, b| a + b)?,
        Sub => binary_op(ctx, span, |a, b| a - b)?,
        Mul => binary_op(ctx, span, |a, b| a * b)?,
        Div => binary_op(ctx, span, |a, b| a / b)?,
        Neg => unary_op(ctx, span, |x| -x)?,
        BitAnd => binary_op(ctx, span, |a, b| a & b)?,
        BitOr => binary_op(ctx, span, |a, b| a | b)?,
        BitNot => unary_op(ctx, span, |x| !x)?,
        Gt => binary_op(ctx, span, |a, b| if a > b { -1 } else { 0 })?,
        Eq => binary_op(ctx, span, |a, b| if a == b { -1 } else { 0 })?,
        Lambda(instructions) => ctx.stack.push(StoredValue(StoredLambda(instructions))),
        Execute => {
            let lambda = match ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack is empty"))?
            {
                StoredValue(StoredLambda(v)) => v,
                other => {
                    return Err(runtime_error!(
                        "Expected lambda for Execute, got {}",
                        other.type_name()
                    ));
                }
            };
            run_instructions(lambda, ctx)?;
        }
        ConditionalExecute(vec) => {
            let condition = pop_int(ctx, span)?;
            if condition != 0 {
                run_instructions(vec, ctx)?;
            }
        }
        WhileLoop(condition, body) => loop {
            run_instructions(&condition, ctx)?;
            let condition_result = pop_int(ctx, span)?;
            if condition_result == 0 {
                break;
            }
            run_instructions(&body, ctx)?;
        },
        Store => {
            let reference = match ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack is empty"))?
            {
                VariableReference(v) => v,
                _ => {
                    return Err(runtime_error!("Store (:) must be preceded by a name"));
                }
            };
            let value = match ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack only has 1 value"))?
            {
                StoredValue(v) => v,
                VariableReference(_) => {
                    return Err(runtime_error!("Names cannot be stored in names"));
                }
            };
            ctx.global_scope.insert(reference, value);
        }
        Fetch => {
            let reference = match ctx
                .stack
                .pop()
                .ok_or_else(error_factory!("Stack is empty"))?
            {
                VariableReference(v) => v,
                _ => {
                    return Err(runtime_error!("Fetch (;) must be preceded by a name"));
                }
            };
            let value = ctx
                .global_scope
                .get(&reference)
                .ok_or_else(|| runtime_error!("Name {reference} not found in global scope"))?
                .clone();
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
            let value = pop_int(ctx, span)?;
            (*ctx.on_output)(
                &std::char::from_u32(value as u32)
                    .ok_or_else(error_factory!("Can't output value {} as char", value))?
                    .to_string(),
            );
        }
        WriteStr(s) => (*ctx.on_output)(&s),
        WriteInt => {
            let value = pop_int(ctx, span)?;
            (*ctx.on_output)(&value.to_string());
        }
        Flush => {}
    };

    Ok(())
}

fn pop_two(
    ctx: &mut FalseContext,
    span: SimpleSpan,
) -> Result<(i32, i32), InterpreterRuntimeError> {
    Ok((pop_int(ctx, span)?, pop_int(ctx, span)?))
}

fn pop_int(ctx: &mut FalseContext, span: SimpleSpan) -> Result<i32, InterpreterRuntimeError> {
    let head = ctx
        .stack
        .pop()
        .ok_or_else(|| InterpreterRuntimeError::new(span, "Stack is empty".to_string()))?;
    match head {
        FalseStackEntry::StoredValue(FalseStoreableValue::StoredInteger(i)) => Ok(i),
        other => Err(InterpreterRuntimeError::new(
            span,
            format!("Expected Integer on stack, got {}", other.type_name()),
        )),
    }
}

fn binary_op(
    ctx: &mut FalseContext,
    span: SimpleSpan,
    op: fn(i32, i32) -> i32,
) -> Result<(), InterpreterRuntimeError> {
    let (a, b) = pop_two(ctx, span)?;
    ctx.stack.push(FalseStackEntry::StoredValue(
        FalseStoreableValue::StoredInteger(op(b, a)),
    ));
    Ok(())
}

fn unary_op(
    ctx: &mut FalseContext,
    span: SimpleSpan,
    op: fn(i32) -> i32,
) -> Result<(), InterpreterRuntimeError> {
    let a = pop_int(ctx, span)?;
    ctx.stack.push(FalseStackEntry::StoredValue(
        FalseStoreableValue::StoredInteger(op(a)),
    ));
    Ok(())
}

fn default_read_input() -> Option<u8> {
    // Read 1 character from stdin
    eprint!("Input character: ");
    let mut buffer = [0; 1];
    std::io::stdin().read_exact(&mut buffer).ok()?;
    eprintln!();
    Some(buffer[0])
}

fn default_output(s: &str) {
    print!("{s}");
}
