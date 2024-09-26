use chumsky::{input::MapExtra, span::SimpleSpan};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Spanned<T>(T, SimpleSpan<usize>);
impl Spanned<FalseInstruction> {
    pub fn new(instruction: FalseInstruction, span: SimpleSpan<usize>) -> Self {
        Self(instruction, span)
    }
    pub fn map_extra<'a, E>(
        instruction: FalseInstruction,
        extra: &mut MapExtra<'a, '_, &'a str, E>,
    ) -> Self
    where
        E: chumsky::extra::ParserExtra<'a, &'a str>,
    {
        Self(instruction, extra.span())
    }

    pub fn instruction(&self) -> &FalseInstruction {
        &self.0
    }

    pub fn span(&self) -> SimpleSpan<usize> {
        self.1
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FalseInstruction {
    Name(char),
    PushInt(i32),
    PushChar(u8),
    Dup,
    Drop,
    Swap,
    Rot,
    Pick,
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    BitAnd,
    BitOr,
    BitNot,
    Gt,
    Eq,
    Lambda(Vec<Spanned<FalseInstruction>>),
    Execute,
    ConditionalExecute(Vec<Spanned<FalseInstruction>>),
    WhileLoop(
        Vec<Spanned<FalseInstruction>>,
        Vec<Spanned<FalseInstruction>>,
    ),
    Store,
    Fetch,
    ReadChar,
    WriteChar,
    WriteStr(String),
    WriteInt,
    Flush,
}
