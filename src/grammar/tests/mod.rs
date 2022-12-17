use super::*;
use Instruction::*;

mod parser;

#[test]
fn parse_single_argument() {
    let expected = Argument("a".into());

    assert_eq!(expected, Instruction::try_from("a").unwrap());
}

#[test]
fn parse_single_nested_argument() {
    let expected = Argument("a".into());

    assert_eq!(expected, Instruction::try_from("(a)").unwrap());
}

#[test]
fn parse_infix() {
    let left = Argument("a".to_string());
    let right = Argument("b".to_string());
    let expected = Xor(Box::new(left), Box::new(right));

    assert_eq!(expected, Instruction::try_from("a + b").unwrap());
}

#[test]
fn parse_nested_infix() {
    let left = Argument("a".to_string());
    let right = Argument("b".to_string());
    let expected = Xor(Box::new(left), Box::new(right));

    assert_eq!(expected, Instruction::try_from("(a + b)").unwrap());
}

#[test]
fn parse_serial_infix() {
    let left = Argument("a".to_string());
    let right = Argument("b".to_string());
    let left = Xor(Box::new(left), Box::new(right));
    let right = Argument("c".to_string());
    let expected = Equals(Box::new(left), Box::new(right));

    assert_eq!(expected, Instruction::try_from("(a + b) = c").unwrap());
}

#[test]
fn parse_complex() {
    let a = Argument("a".to_string());
    let b = Argument("b".to_string());
    let xor = Xor(Box::new(a), Box::new(b));
    let not_xor = Not(Box::new(xor));
    let c = Argument("c".to_string());
    let not_c = Not(Box::new(c));
    let equals = Equals(Box::new(not_xor), Box::new(not_c));
    let x = Argument("x".to_string());
    let expected = Conditional(Box::new(equals), Box::new(x));

    assert_eq!(
        expected,
        Instruction::try_from("!(a + b) = !c -> x").unwrap()
    );
}
