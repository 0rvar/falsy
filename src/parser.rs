use chumsky::prelude::*;

use crate::ast::FalseInstruction;

// fn parser<'a>() -> impl Parser<'a, &'a str, Vec<FalseInstruction>, extra::Err<Rich<'a, char>>> {
//     recursive(|value| {
//         let int = text::int(10).map(|s: &str| FalseInstruction::PushInt(s.parse().unwrap()));

//         let char_lit = just('\'')
//             .ignore_then(any())
//             .map(FalseInstruction::PushChar);

//         let string = none_of("\\\"")
//             .ignored()
//             .repeated()
//             .to_slice()
//             .map(ToString::to_string)
//             .delimited_by(just('"'), just('"'))
//             .boxed()
//             .map(FalseInstruction::WriteStr);

//         let stack = choice((
//             just('$').to(FalseInstruction::Dup),
//             just('%').to(FalseInstruction::Drop),
//             just('\\').to(FalseInstruction::Swap),
//             just('@').to(FalseInstruction::Rot),
//             just('ø').to(FalseInstruction::Pick),
//         ));

//         let arithmetic = choice((
//             just('+').to(FalseInstruction::Add),
//             just('-').to(FalseInstruction::Sub),
//             just('*').to(FalseInstruction::Mul),
//             just('/').to(FalseInstruction::Div),
//             just('_').to(FalseInstruction::Neg),
//             just('&').to(FalseInstruction::BitAnd),
//             just('|').to(FalseInstruction::BitOr),
//             just('~').to(FalseInstruction::BitNot),
//         ));

//         let comparison = choice((
//             just('>').to(FalseInstruction::Gt),
//             just('=').to(FalseInstruction::Eq),
//         ));

//         let lambda = value.delimited_by(just("["), just("]"))
//           .map(FalseInstruction::Lambda)

//         let conditional = lambda
//             .clone()
//             .then(just('?').ignore_then(lambda.clone()))
//             .map(|(cond, body)| FalseInstruction::ConditionalExecute(vec![cond], vec![body]));

//         let while_loop = lambda
//             .clone()
//             .then(lambda.clone())
//             .map(|(cond, body)| FalseInstruction::WhileLoop(vec![cond], vec![body]));

//         let flow_control = choice((
//             just('!').to(FalseInstruction::Execute),
//             conditional,
//             while_loop,
//         ));

//         let names = just(':')
//             .to(FalseInstruction::Store)
//             .or(just(';').to(FalseInstruction::Fetch));

//         let io = choice((
//             just('^').to(FalseInstruction::ReadChar),
//             just(',').to(FalseInstruction::WriteChar),
//             just('.').to(FalseInstruction::WriteInt),
//             just('ß').to(FalseInstruction::Flush),
//         ));

//         let instr = choice((
//             int,
//             char_lit,
//             string,
//             stack,
//             arithmetic,
//             comparison,
//             lambda,
//             flow_control,
//             names,
//             io,
//         ));

//         instr.repeated().then_ignore(end())
//     })
// }

pub fn parse(input: &str) -> ParseResult<Vec<FalseInstruction>, Rich<char>> {
    // parser().parse(input)
    todo!();
}
