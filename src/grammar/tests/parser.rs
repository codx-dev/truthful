use std::collections::HashSet;

use super::*;
use pest::Parser;

#[test]
fn parse_not() {
    expect_rule("!", inner::Rule::not, "!");
    for expr in permutate_case("not") {
        expect_rule(&expr, inner::Rule::not, &expr);
    }
}

#[test]
fn parse_and() {
    expect_rule("^", inner::Rule::and, "^");
    for expr in permutate_case("and") {
        expect_rule(&expr, inner::Rule::and, &expr);
    }
}

#[test]
fn parse_or() {
    expect_rule("v", inner::Rule::or, "v");
    for expr in permutate_case("or") {
        expect_rule(&expr, inner::Rule::or, &expr);
    }
}

#[test]
fn parse_xor() {
    expect_rule("+", inner::Rule::xor, "+");
    for expr in permutate_case("xor") {
        expect_rule(&expr, inner::Rule::xor, &expr);
    }
}

#[test]
fn parse_cond() {
    expect_rule("->", inner::Rule::cond, "->");
    for expr in permutate_case("cond") {
        expect_rule(&expr, inner::Rule::cond, &expr);
    }
}

#[test]
fn parse_bicond() {
    expect_rule("<->", inner::Rule::bicond, "<->");
    for expr in permutate_case("bicond") {
        expect_rule(&expr, inner::Rule::bicond, &expr);
    }
}

#[test]
fn parse_equals() {
    expect_rule("=", inner::Rule::equals, "=");
    for expr in permutate_case("equals") {
        expect_rule(&expr, inner::Rule::equals, &expr);
    }
}

#[test]
fn parse_identifier() {
    expect_rule("a", inner::Rule::identifier, "a");
    expect_rule("xyz", inner::Rule::identifier, "xyz");
    expect_rule("xyz_ab-c0", inner::Rule::identifier, "xyz_ab-c0");
    expect_rule("\"xyz abc\"", inner::Rule::identifier, "\"xyz abc\"");
    expect_rule("'xyz abc'", inner::Rule::identifier, "'xyz abc'");
}

#[test]
fn parse_term() {
    expect_rule("a", inner::Rule::term, "a");
    expect_rule("(a)", inner::Rule::term, "(a)");
    expect_rule("(a + b)", inner::Rule::term, "(a + b)");
}

#[test]
fn parse_expr() {
    expect_rule("a", inner::Rule::expr, "a");
    expect_rule("(a)", inner::Rule::expr, "(a)");
    expect_rule("a + b", inner::Rule::expr, "a + b");
    expect_rule("(a + b)", inner::Rule::expr, "(a + b)");
    expect_rule("(a + b) = c", inner::Rule::expr, "(a + b) = c");
    expect_rule(
        "!(a + b) = !c -> x",
        inner::Rule::expr,
        "!(a + b) = !c -> x",
    );
}

fn expect_rule<X, Y>(expr: X, rule: inner::Rule, span: Y)
where
    X: AsRef<str>,
    Y: AsRef<str>,
{
    let pair = match inner::Parser::parse(rule, expr.as_ref()) {
        Ok(mut pair) => pair.next().unwrap(),
        Err(e) => panic!("{}", e.to_string()),
    };
    let parsed = pair.as_rule();
    let content = pair.as_str();

    assert_eq!(rule, parsed);
    assert_eq!(span.as_ref(), content);
}

fn permutate_case<X>(expr: X) -> impl IntoIterator<Item = String>
where
    X: AsRef<str>,
{
    let expr = expr.as_ref();
    let mut len = 1usize << expr.len();
    let mut result = vec![String::from(""); len];

    expr.chars().for_each(|c| {
        len /= 2;
        let mut pair = [
            c.to_lowercase().next().unwrap(),
            c.to_uppercase().next().unwrap(),
        ];

        result.iter_mut().fold(0, |mut j, r| {
            if j == len {
                let y = pair[0];
                pair[0] = pair[1];
                pair[1] = y;
                j = 1;
            } else {
                j += 1;
            }
            r.push(pair[0]);
            j
        });
    });

    result
}

#[test]
fn permutate_case_sanity() {
    assert_eq!(
        permutate_case("t").into_iter().collect::<HashSet<_>>(),
        HashSet::from_iter(vec!["t".to_string(), "T".to_string(),])
    );
    assert_eq!(
        permutate_case("ot").into_iter().collect::<HashSet<_>>(),
        HashSet::from_iter(vec![
            "ot".to_string(),
            "oT".to_string(),
            "Ot".to_string(),
            "OT".to_string(),
        ])
    );
    assert_eq!(
        permutate_case("not").into_iter().collect::<HashSet<_>>(),
        HashSet::from_iter(vec![
            "not".to_string(),
            "noT".to_string(),
            "nOt".to_string(),
            "nOT".to_string(),
            "Not".to_string(),
            "NoT".to_string(),
            "NOt".to_string(),
            "NOT".to_string(),
        ])
    );
    assert_eq!(
        permutate_case("pair").into_iter().collect::<HashSet<_>>(),
        HashSet::from_iter(vec![
            "pair".to_string(),
            "paiR".to_string(),
            "paIr".to_string(),
            "paIR".to_string(),
            "pAir".to_string(),
            "pAiR".to_string(),
            "pAIr".to_string(),
            "pAIR".to_string(),
            "Pair".to_string(),
            "PaiR".to_string(),
            "PaIr".to_string(),
            "PaIR".to_string(),
            "PAir".to_string(),
            "PAiR".to_string(),
            "PAIr".to_string(),
            "PAIR".to_string(),
        ])
    );
}
