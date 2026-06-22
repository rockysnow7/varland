use crate::environment::Value;
use nom::{
    IResult,
    Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{alpha1, char, digit0, digit1, satisfy, space0, one_of},
    combinator::{opt, value},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, separated_pair},
};

fn parse_bool(input: &str) -> IResult<&str, Value> {
    alt((
        value(Value::Bool(true), tag_no_case("true")),
        value(Value::Bool(false), tag_no_case("false")),
    )).parse(input)
}

fn parse_int(input: &str) -> IResult<&str, Value> {
    (
        opt(alt((char('-'), char('+')))),
        digit1,
    ).parse(input).map(|(rest, (sign, value))| {
        let positive = match sign {
            Some('-') => false,
            Some('+') => true,
            None => true,
            _ => unreachable!(),
        };
        let mut value = value.parse::<i64>().unwrap();
        if !positive {
            value = -value;
        }
        (rest, Value::Int(value))
    })
}

fn parse_float(input: &str) -> IResult<&str, Value> {
    (
        opt(alt((char('-'), char('+')))),
        separated_pair(
            digit0,
            tag("."),
            digit0,
        ),
    ).parse(input).map(|(rest, (sign, (integer, decimal)))| {
        let positive = match sign {
            Some('-') => false,
            Some('+') => true,
            None => true,
            _ => unreachable!(),
        };
        let mut value = format!("{integer}.{decimal}").parse::<f64>().unwrap();
        if !positive {
            value = -value;
        }

        (rest, Value::Float(value))
    })
}

fn parse_string(input: &str) -> IResult<&str, Value> {
    delimited(
        char('"'),
        many0(satisfy(|c| c != '"')),
        char('"'),
    ).parse(input).map(|(rest, value)| (rest, Value::String(value.iter().collect())))
}

fn parse_list(input: &str) -> IResult<&str, Value> {
    delimited(
        char('['),
        separated_list0(
            char(','),
            delimited(space0, parse, space0),
        ),
        char(']'),
    ).parse(input).map(|(rest, values)| (rest, Value::List(values)))
}

fn parse_name(input: &str) -> IResult<&str, String> {
    many1(satisfy(|c| c.is_ascii_alphanumeric() || c == '_'))
        .parse(input)
        .map(|(rest, value)| (rest, value.iter().collect()))
}

fn parse_function_call(input: &str) -> IResult<&str, Value> {
    (
        delimited(space0, parse_name, space0),
        delimited(
            char('('),
            separated_list0(char(','), delimited(space0, parse, space0)),
            char(')'),
        ),
    ).parse(input).map(|(rest, (function_name, arguments))| (rest, Value::FunctionCall { function_name, arguments }))
}

fn parse_usize_1_or_more(input: &str) -> IResult<&str, usize> {
    (one_of("123456789"), digit0).parse(input).map(|(rest, (a, b))| {
        let value = format!("{a}{b}").parse::<usize>().unwrap();

        (rest, value)
    })
}

fn parse_clone_cell(input: &str) -> IResult<&str, Value> {
    (alpha1, parse_usize_1_or_more).parse(input).map(|(rest, (column, row))| {
        let col = column.chars().fold(0usize, |col, c| col * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1)) - 1;
        let row = row - 1;

        (rest, Value::CloneCell { col, row })
    })
}

fn parse_clone_cell_range(input: &str) -> IResult<&str, Value> {
    separated_pair(parse_clone_cell, char(':'), parse_clone_cell)
        .parse(input)
        .map(|(rest, (start, end))| {
            let Value::CloneCell { col: start_col, row: start_row } = start else { unreachable!() };
            let Value::CloneCell { col: end_col, row: end_row } = end else { unreachable!() };

            let range = Value::CloneCellRange {
                start_col,
                start_row,
                end_col,
                end_row,
            };

            (rest, range)
        })
}

pub fn parse(input: &str) -> IResult<&str, Value> {
    alt((
        parse_function_call,
        parse_clone_cell_range,
        parse_clone_cell,
        parse_list,
        parse_float,
        parse_int,
        parse_bool,
        parse_string,
    )).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_function_call() {
        assert_eq!(parse("sum(1, 2, 3)"), Ok(("", Value::FunctionCall { function_name: "sum".to_string(), arguments: vec![Value::Int(1), Value::Int(2), Value::Int(3)] })));
        assert_eq!(parse("a1_()"), Ok(("", Value::FunctionCall { function_name: "a1_".to_string(), arguments: vec![] })));
    }

    #[test]
    fn test_parse_clone_cell_range() {
        assert_eq!(parse("A1:B2"), Ok(("", Value::CloneCellRange { start_col: 0, start_row: 0, end_col: 1, end_row: 1 })));
        assert_eq!(parse("aA1:Bz2"), Ok(("", Value::CloneCellRange { start_col: 26, start_row: 0, end_col: 77, end_row: 1 })));
    }

    #[test]
    fn test_parse_clone_cell() {
        assert_eq!(parse("A1"), Ok(("", Value::CloneCell { col: 0, row: 0 })));
        assert_eq!(parse("z1"), Ok(("", Value::CloneCell { col: 25, row: 0 })));
        assert_eq!(parse("AA10"), Ok(("", Value::CloneCell { col: 26, row: 9 })));
    }
    
    #[test]
    fn test_parse_list() {
        assert_eq!(parse("[ 1, 2, 3]"), Ok(("", Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]))));
        assert_eq!(
            parse("[1.0, \"hello\", [1]]"),
            Ok(("", Value::List(vec![Value::Float(1.0), Value::String("hello".to_string()), Value::List(vec![Value::Int(1)])]))),
        );
        assert_eq!(parse("[]"), Ok(("", Value::List(vec![]))));
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse("1.23"), Ok(("", Value::Float(1.23))));
        assert_eq!(parse("-10.0"), Ok(("", Value::Float(-10.0))));
        assert_eq!(parse("+1.23"), Ok(("", Value::Float(1.23))));
        assert_eq!(parse("+.01"), Ok(("", Value::Float(0.01))));
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse("123"), Ok(("", Value::Int(123))));
        assert_eq!(parse("-456"), Ok(("", Value::Int(-456))));
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse("true"), Ok(("", Value::Bool(true))));
        assert_eq!(parse("false"), Ok(("", Value::Bool(false))));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse("\"Hello, world!\""), Ok(("", Value::String("Hello, world!".to_string()))));
    }
}
