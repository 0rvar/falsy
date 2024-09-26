use chumsky::prelude::*;

use crate::ast::{FalseInstruction, Spanned};

fn parser<'a>(
) -> impl Parser<'a, &'a str, Vec<Spanned<FalseInstruction>>, extra::Err<Rich<'a, char>>> {
    recursive(|value| {
        let comment = just('{')
            .then(any().and_is(just('}').not()).repeated())
            .then(just('}'))
            .padded()
            .repeated();

        let int = text::int(10).map(|s: &str| FalseInstruction::PushInt(s.parse().unwrap()));

        // 'a is a char literal for a (no trailing ')
        let char_lit = just('\'')
            .ignored()
            .then(any())
            .try_map(|(_, c): (_, char), span| {
                let character = c.to_string().as_bytes().first().cloned();
                match character {
                    Some(c) => Ok(FalseInstruction::PushChar(c)),
                    None => Err(Rich::custom(span, "Invalid character")),
                }
            });

        let string = none_of("\"")
            .repeated()
            .collect()
            .delimited_by(just('"'), just('"'))
            .map(FalseInstruction::WriteStr);

        let variable_name = false_name().map(FalseInstruction::Name);

        let stack = choice((
            just('$').to(FalseInstruction::Dup),
            just('%').to(FalseInstruction::Drop),
            just('\\').to(FalseInstruction::Swap),
            just('@').to(FalseInstruction::Rot),
            just('ø').to(FalseInstruction::Pick),
        ));

        let arithmetic = choice((
            just('+').to(FalseInstruction::Add),
            just('-').to(FalseInstruction::Sub),
            just('*').to(FalseInstruction::Mul),
            just('/').to(FalseInstruction::Div),
            just('_').to(FalseInstruction::Neg),
            just('&').to(FalseInstruction::BitAnd),
            just('|').to(FalseInstruction::BitOr),
            just('~').to(FalseInstruction::BitNot),
        ));

        let comparison = choice((
            just('>').to(FalseInstruction::Gt),
            just('=').to(FalseInstruction::Eq),
        ));

        let subexpression = value.delimited_by(
            just('['),
            just(']')
                .ignored()
                .recover_with(via_parser(end()))
                .recover_with(skip_then_retry_until(any().ignored(), end())),
        );

        let lambda = subexpression.clone().map(FalseInstruction::Lambda);

        let conditional = subexpression
            .clone()
            .then_ignore(just('?'))
            .map(FalseInstruction::ConditionalExecute);

        let while_loop = subexpression
            .clone()
            .then(subexpression.clone())
            .then_ignore(just('#'))
            .map(|(cond, body)| FalseInstruction::WhileLoop(cond, body));

        let flow_control = choice((
            just('!').to(FalseInstruction::Execute),
            conditional,
            while_loop,
        ));

        let store_fetch = just(':')
            .to(FalseInstruction::Store)
            .or(just(';').to(FalseInstruction::Fetch));

        let io = choice((
            just('^').to(FalseInstruction::ReadChar),
            just(',').to(FalseInstruction::WriteChar),
            just('.').to(FalseInstruction::WriteInt),
            just('ß').to(FalseInstruction::Flush),
        ));

        let instr = choice((
            int,
            char_lit,
            string,
            variable_name,
            store_fetch,
            stack,
            arithmetic,
            comparison,
            flow_control,
            lambda,
            io,
        ));

        instr
            .map_with(Spanned::map_extra)
            .padded_by(comment)
            .padded()
            .repeated()
            .collect()
    })
}

pub fn parse(input: &str) -> ParseResult<Vec<Spanned<FalseInstruction>>, Rich<char>> {
    parser().parse(input)
}

pub fn false_name<'a, C, I, E>() -> impl Parser<'a, I, C, E> + Copy
where
    C: text::Char,
    I: chumsky::input::ValueInput<'a> + Input<'a, Token = C>,
    E: extra::ParserExtra<'a, I>,
{
    any()
        // Use try_map over filter to get a better error on failure
        .try_map(move |c: C, span| {
            let as_char = c.to_char();
            if ('a'..='z').contains(&as_char) {
                Ok(c)
            } else {
                Err(chumsky::error::Error::expected_found(
                    [],
                    Some(chumsky::util::MaybeRef::Val(c)),
                    span,
                ))
            }
        })
}
