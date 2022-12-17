use super::Instruction;
use core::fmt;
use inner::Rule;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};

#[cfg(test)]
mod tests;

mod inner {
    #[derive(pest_derive::Parser)]
    #[grammar = "./src/grammar/vco.pest"]
    pub struct Parser;
}

impl TryFrom<&str> for Instruction {
    type Error = String;

    fn try_from(program: &str) -> Result<Self, Self::Error> {
        let mut pairs =
            inner::Parser::parse(Rule::statement, program).map_err(|e| e.to_string())?;

        let pair = fetch_pair(&mut pairs)?;
        let expr = fetch_pair(&mut pair.into_inner())?;

        fetch_expr(&mut expr.into_inner()).map(Self::optimize)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instruction::*;
        match self {
            True => write!(f, "1"),
            False => write!(f, "0"),
            Argument(a) => write!(f, "\"{a}\""),
            Not(i) => write!(f, "!{i}"),
            And(l, r) => write!(f, "({l} ^ {r})"),
            Or(l, r) => write!(f, "({l} v {r})"),
            Xor(l, r) => write!(f, "({l} + {r})"),
            Conditional(l, r) => write!(f, "({l} -> {r})"),
            Biconditional(l, r) => write!(f, "({l} <-> {r})"),
            Equals(l, r) => write!(f, "({l} = {r})"),
        }
    }
}

fn fetch_pair<'a>(pairs: &mut Pairs<'a, Rule>) -> Result<Pair<'a, Rule>, String> {
    pairs
        .next()
        .ok_or_else(|| "the grammar is inconsistent: a pair was expected".to_string())
}

fn fetch_term(pairs: &mut Pairs<'_, Rule>) -> Result<Instruction, String> {
    let mut pair = fetch_pair(pairs)?;

    let mut unary_not = false;
    while let Rule::not = pair.as_rule() {
        unary_not = !unary_not;
        pair = fetch_pair(pairs)?
    }

    let instruction = match pair.as_rule() {
        Rule::identifier => {
            let mut identifier = pair.as_str().to_string();
            if identifier.starts_with('"') {
                identifier = identifier
                    .strip_prefix('"')
                    .unwrap_or(identifier.as_str())
                    .strip_suffix('"')
                    .unwrap_or(identifier.as_str())
                    .to_string();
            } else if identifier.starts_with('\'') {
                identifier = identifier
                    .strip_prefix('\'')
                    .unwrap_or(identifier.as_str())
                    .strip_suffix('\'')
                    .unwrap_or(identifier.as_str())
                    .to_string();
            }
            Instruction::Argument(identifier)
        }
        Rule::expr => fetch_expr(&mut pair.into_inner())?,
        _ => return Err("inconsistent grammar".to_string()),
    };

    if unary_not {
        Ok(Instruction::Not(Box::new(instruction)))
    } else {
        Ok(instruction)
    }
}

fn fetch_infix_rule(pairs: &mut Pairs<'_, Rule>) -> Result<Option<Rule>, String> {
    let pair = match pairs.next() {
        Some(p) => p,
        None => return Ok(None),
    };

    let rule = pair.as_rule();
    use Rule::*;
    match rule {
        and | or | xor | cond | bicond | equals => Ok(Some(rule)),
        _ => Err("invalid grammar: expected infix rule".to_string()),
    }
}

fn fetch_expr(pairs: &mut Pairs<'_, Rule>) -> Result<Instruction, String> {
    let pair = fetch_pair(pairs)?;
    let mut term = fetch_term(&mut pair.into_inner())?;

    while let Some(infix) = fetch_infix_rule(pairs)? {
        let lhs = Box::new(term);

        let rhs = fetch_pair(pairs)?;
        let rhs = fetch_term(&mut rhs.into_inner())?;
        let rhs = Box::new(rhs);

        use Instruction::*;
        term = match infix {
            Rule::and => And(lhs, rhs),
            Rule::or => Or(lhs, rhs),
            Rule::xor => Xor(lhs, rhs),
            Rule::cond => Conditional(lhs, rhs),
            Rule::bicond => Biconditional(lhs, rhs),
            Rule::equals => Equals(lhs, rhs),
            _ => return Err("internal error computing infix".to_string()),
        };
    }

    Ok(term)
}
