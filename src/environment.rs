mod functions;

use crate::utils::{coords_to_string, Set};
use functions::{builtin_functions, Function};
use std::{collections::HashMap, fmt::Display, ops::BitOr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Null,
    Bool,
    Int,
    Float,
    String,
    List(Option<Box<ValueType>>), // None if the list is empty
    Union(Set<ValueType>),
}

impl ValueType {
    /// Returns true if `self` is an improper subset of `other`.
    pub fn is_subset_of(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }
        // if let Self::Union(types) = other {
        //     return types.iter().any(|t| self.is_subset_of(t));
        // }

        match (self, other) {
            (Self::Union(types), Self::Union(other_types)) => types.is_subset_of(other_types),
            (_, Self::Union(other_types)) => other_types.iter().any(|t| self.is_subset_of(t)),
            (Self::List(Some(inner)), Self::List(Some(other_inner))) => inner.is_subset_of(other_inner),
            (Self::List(None), Self::List(_)) => true,
            (Self::List(Some(_)), Self::List(None)) => false,
            _ => false,
        }
    }

    /// Returns a list of the given type (convenience function).
    pub fn list_of(inner: Self) -> Self {
        Self::List(Some(Box::new(inner)))
    }

    /// Returns a union of the given types (convenience function).
    pub fn union_of(types: impl IntoIterator<Item = Self>) -> Self {
        Self::Union(Set::from_iter(types))
    }

    /// Returns the union of `self` and `other`. If `self == other`, returns `self`. You can also use the `|` operator.
    pub fn or(&self, other: &Self) -> Self {
        if self == other {
            return self.clone();
        }

        let self_types = if let Self::Union(types) = self { types.clone() } else { Set::from_iter([self.clone()]) };
        let other_types = if let Self::Union(types) = other { types.clone() } else { Set::from_iter([other.clone()]) };
        Self::Union(self_types | other_types)
    }
}

impl BitOr for ValueType {
    type Output = Self;

    /// Returns the union of `self` and `other`. Overload of the `ValueType::or` method.
    fn bitor(self, other: Self) -> Self::Output {
        self.or(&other)
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "Null"),
            Self::Bool => write!(f, "Bool"),
            Self::Int => write!(f, "Int"),
            Self::Float => write!(f, "Float"),
            Self::String => write!(f, "String"),
            Self::List(Some(inner)) => write!(f, "[{inner}]"),
            Self::List(None) => write!(f, "[]"),
            Self::Union(types) => write!(
                f,
                "{}",
                types.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(" | "),
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    FunctionCall {
        function_name: String,
        arguments: Vec<Value>,
    },
    CloneCell {
        col: usize,
        row: usize,
    },
    CloneCellRange {
        start_col: usize,
        start_row: usize,
        end_col: usize,
        end_row: usize,
    },
}

impl Value {
    pub fn type_(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
            Value::List(values) => {
                if values.is_empty() {
                    return ValueType::List(None);
                }

                let mut inner_types = Set::new();
                for value in values {
                    inner_types.insert(value.type_());
                }

                if inner_types.len() == 1 {
                    ValueType::list_of(inner_types.iter().next().unwrap().clone())
                } else {
                    ValueType::list_of(ValueType::Union(inner_types))
                }
            },
            Value::FunctionCall { .. } => unreachable!("FunctionCall should be evaluated to a concrete value first"),
            Value::CloneCell { .. } => unreachable!("CloneCell should be evaluated to a concrete value first"),
            Value::CloneCellRange { .. } => unreachable!("CloneCellRange should be evaluated to a concrete value first"),
        }
    }
}

pub struct Environment {
    functions: HashMap<String, Function>,
    raw_table: Vec<Vec<Value>>,
    evaluated_table: Vec<Vec<Result<Value, String>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            functions: builtin_functions(),
            raw_table: Vec::new(),
            evaluated_table: Vec::new(),
        }
    }

    pub fn get_value_raw(&self, col: usize, row: usize) -> Value {
        if col >= self.raw_table.len() {
            return Value::Null;
        }
        if row >= self.raw_table[col].len() {
            return Value::Null;
        }
        self.raw_table[col][row].clone()
    }

    pub fn set_value_raw(&mut self, col: usize, row: usize, value: Value) {
        if col >= self.raw_table.len() {
            self.raw_table.resize(col + 1, Vec::new());
        }
        if row >= self.raw_table[col].len() {
            self.raw_table[col].resize(row + 1, Value::Null);
        }
        self.raw_table[col][row] = value;
    }

    pub fn get_value_evaluated(&self, col: usize, row: usize) -> Result<Value, String> {
        if col >= self.evaluated_table.len() {
            return Ok(Value::Null);
        }
        if row >= self.evaluated_table[col].len() {
            return Ok(Value::Null);
        }
        self.evaluated_table[col][row].clone()
    }

    fn set_value_evaluated(&mut self, col: usize, row: usize, value: Result<Value, String>) {
        if col >= self.evaluated_table.len() {
            self.evaluated_table.resize(col + 1, Vec::new());
        }
        if row >= self.evaluated_table[col].len() {
            self.evaluated_table[col].resize(row + 1, Ok(Value::Null));
        }
        self.evaluated_table[col][row] = value;
    }

    fn evaluate_function_call(&self, function_name: &str, arguments: &[Value], source_col: usize, source_row: usize) -> Result<Value, String> {
        let arguments_evaluated = arguments
            .iter()
            .map(|arg| self.evaluate_raw_value(arg, source_col, source_row))
            .collect::<Result<Vec<Value>, String>>()?;
        let function = self.functions.get(function_name).unwrap();
        function.call(&arguments_evaluated)
    }

    fn evaluate_raw_value(&self, raw_value: &Value, source_col: usize, source_row: usize) -> Result<Value, String> {
        match raw_value {
            Value::Null => Ok(Value::Null),
            Value::Int(value) => Ok(Value::Int(*value)),
            Value::Float(value) => Ok(Value::Float(*value)),
            Value::String(value) => Ok(Value::String(value.clone())),
            Value::Bool(value) => Ok(Value::Bool(*value)),
            Value::List(value) => {
                let values = value
                    .iter()
                    .map(|v| self.evaluate_raw_value(v, source_col, source_row))
                    .collect::<Result<Vec<Value>, String>>()?;
                Ok(Value::List(values))
            },
            Value::FunctionCall { function_name, arguments } => self.evaluate_function_call(function_name, arguments, source_col, source_row),
            Value::CloneCell { col, row } => if *col > source_col || *col == source_col && *row >= source_row {
                Err(format!("Cannot clone cell {} from source cell {} as the former comes after the latter", coords_to_string(*col, *row), coords_to_string(source_col, source_row)))
            } else {
                self.get_value_evaluated(*col, *row)
            },
            Value::CloneCellRange { start_col, start_row, end_col, end_row } => {
                if *start_col > *end_col {
                    return Err(format!("Start column {start_col} is greater than end column {end_col}"));
                }
                if *start_row > *end_row {
                    return Err(format!("Start row {start_row} is greater than end row {end_row}"));
                }
                if *end_col > source_col || *end_col == source_col && *end_row >= source_row {
                    return Err(format!(
                        "Cannot clone cell range {}:{} as it contains cells that come after the source cell ({})",
                        coords_to_string(*start_col, *start_row),
                        coords_to_string(*end_col, *end_row),
                        coords_to_string(source_col, source_row),
                    ));
                }

                let mut values = Vec::new();
                for col in *start_col..=*end_col {
                    for row in *start_row..=*end_row {
                        let value = self.get_value_evaluated(col, row)?;
                        if value == Value::Null {
                            continue;
                        }
                        values.push(value);
                    }
                }
                Ok(Value::List(values))
            }
        }
    }

    fn evaluate_cell(&mut self, row: usize, column: usize) {
        let raw_value = self.get_value_raw(row, column);
        let evaluated_value = self.evaluate_raw_value(&raw_value, row, column);
        self.set_value_evaluated(row, column, evaluated_value);
    }

    pub fn evaluate_all(&mut self) {
        for row in 0..self.raw_table.len() {
            for column in 0..self.raw_table[row].len() {
                self.evaluate_cell(row, column);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_is_subset_of() {
        assert!(ValueType::Int.is_subset_of(&ValueType::Int));
        assert!(ValueType::Int.is_subset_of(&(ValueType::Int | ValueType::Float)));
        assert!(ValueType::list_of(ValueType::Int | ValueType::Float)
            .is_subset_of(&ValueType::list_of(ValueType::Int | ValueType::Float | ValueType::String)));
        assert!((ValueType::Int | ValueType::Float).is_subset_of(&(ValueType::Int | ValueType::Float | ValueType::String)));
    }
}
