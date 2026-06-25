mod functions;

use crate::{
    parser::parse,
    utils::{Set, coords_to_string},
};
use functions::{Function, builtin_functions};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt::Display, ops::BitOr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Any,
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
        if other == &Self::Any {
            return true;
        }
        if self == other {
            return true;
        }

        match (self, other) {
            (Self::Union(types), Self::Union(other_types)) => types.is_subset_of(other_types),
            (Self::Union(types), other) => types.iter().all(|t| t.is_subset_of(other)),
            (_, Self::Union(other_types)) => other_types.iter().any(|t| self.is_subset_of(t)),
            (Self::List(Some(inner)), Self::List(Some(other_inner))) => {
                inner.is_subset_of(other_inner)
            }
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

        let self_types = if let Self::Union(types) = self {
            types.clone()
        } else {
            Set::from_iter([self.clone()])
        };
        let other_types = if let Self::Union(types) = other {
            types.clone()
        } else {
            Set::from_iter([other.clone()])
        };
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
            Self::Any => write!(f, "Any"),
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
                types
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(" | "),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            Self::Null => ValueType::Null,
            Self::Bool(_) => ValueType::Bool,
            Self::Int(_) => ValueType::Int,
            Self::Float(_) => ValueType::Float,
            Self::String(_) => ValueType::String,
            Self::List(values) => {
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
            }
            Self::FunctionCall { .. } => {
                unreachable!("FunctionCall should be evaluated to a concrete value first")
            }
            Self::CloneCell { .. } => {
                unreachable!("CloneCell should be evaluated to a concrete value first")
            }
            Self::CloneCellRange { .. } => {
                unreachable!("CloneCellRange should be evaluated to a concrete value first")
            }
        }
    }

    pub fn infer_from_str(s: &str) -> Self {
        match parse(s) {
            Ok(("", value)) => value,
            _ => Value::String(s.to_string()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, ""),
            Self::Bool(bool) => write!(f, "{bool}"),
            Self::Int(int) => write!(f, "{int}"),
            Self::Float(float) => write!(f, "{float}"),
            Self::String(string) => write!(f, "\"{string}\""),
            Self::List(values) => {
                let values_str = values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "[{values_str}]")
            }
            Self::FunctionCall {
                function_name,
                arguments,
            } => {
                let arguments_str = arguments
                    .iter()
                    .map(|argument| argument.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{function_name}({arguments_str})")
            }
            Self::CloneCell { col, row } => write!(f, "{}", coords_to_string(*col, *row)),
            Self::CloneCellRange {
                start_col,
                start_row,
                end_col,
                end_row,
            } => write!(
                f,
                "{}:{}",
                coords_to_string(*start_col, *start_row),
                coords_to_string(*end_col, *end_row),
            ),
        }
    }
}

pub struct Environment {
    functions: HashMap<String, Function>,
    raw_table: Vec<Vec<Value>>,
    evaluated_table: Vec<Vec<Option<Result<Value, String>>>>, // (col, row) -> option<result> (None if not evaluated yet)
}

impl Environment {
    pub fn new() -> Self {
        Self {
            functions: builtin_functions(),
            raw_table: Vec::new(),
            evaluated_table: Vec::new(),
        }
    }

    /// Loads an `Environment` from a CSV file.
    pub fn new_from_csv(path_to_file: &str) -> Result<Self, csv::Error> {
        let mut reader = csv::ReaderBuilder::new().has_headers(false).from_path(path_to_file)?;

        let records = reader.records().collect::<Result<Vec<csv::StringRecord>, csv::Error>>()?;

        let num_cols = records
            .iter()
            .map(|record| record.len())
            .max()
            .unwrap();
        let num_rows = records.len();

        let mut table = vec![vec![Value::Null; num_rows]; num_cols];
        let headers = records.iter().next().unwrap();
        for (col, header) in headers.iter().enumerate() {
            table[col][0] = Value::String(header.to_string());
        }

        for (row, record) in records.iter().enumerate().skip(1) {
            for (col, value) in record.iter().enumerate() {
                let value = Value::infer_from_str(value);
                table[col][row] = value;
            }
        }

        let mut environment = Self::new();
        environment.raw_table = table;

        Ok(environment)
    }

    /// Returns the number of columns in the environment.
    pub fn num_cols(&self) -> usize {
        self.raw_table.len()
    }

    /// Returns the number of rows in the environment.
    pub fn num_rows(&self) -> usize {
        if self.raw_table.is_empty() { return 0; }

        let max_rows = self.raw_table.iter().map(|row| row.len()).max().unwrap();
        max_rows
    }

    /// Evaluates a function call.
    fn evaluate_function_call(
        &mut self,
        function_name: &str,
        arguments: &[Value],
        source_col: usize,
        source_row: usize,
    ) -> Result<Value, String> {
        let arguments_evaluated = arguments
            .iter()
            .map(|arg| self.evaluate_value(arg, source_col, source_row))
            .collect::<Result<Vec<Value>, String>>()?;
        let function = if let Some(function) = self.functions.get(function_name) {
            function
        } else {
            return Err(format!("Function `{function_name}` not found"));
        };
        function.call(&arguments_evaluated)
    }

    /// Evaluates a raw value.
    fn evaluate_value(
        &mut self,
        raw_value: &Value,
        source_col: usize,
        source_row: usize,
    ) -> Result<Value, String> {
        match raw_value {
            Value::Null => Ok(Value::Null),
            Value::Int(value) => Ok(Value::Int(*value)),
            Value::Float(value) => Ok(Value::Float(*value)),
            Value::String(value) => Ok(Value::String(value.clone())),
            Value::Bool(value) => Ok(Value::Bool(*value)),
            Value::List(value) => {
                let values = value
                    .iter()
                    .map(|v| self.evaluate_value(v, source_col, source_row))
                    .collect::<Result<Vec<Value>, String>>()?;
                Ok(Value::List(values))
            }
            Value::FunctionCall {
                function_name,
                arguments,
            } => self.evaluate_function_call(function_name, arguments, source_col, source_row),
            Value::CloneCell { col, row } => {
                if *col > source_col || *col == source_col && *row >= source_row {
                    Err(format!(
                        "Cannot clone cell {} from source cell {} as the former comes after the latter",
                        coords_to_string(*col, *row),
                        coords_to_string(source_col, source_row)
                    ))
                } else {
                    self.get_evaluated(*col, *row)
                }
            }
            Value::CloneCellRange {
                start_col,
                start_row,
                end_col,
                end_row,
            } => {
                if *start_col > *end_col {
                    return Err(format!(
                        "Start column {start_col} is greater than end column {end_col}"
                    ));
                }
                if *start_row > *end_row {
                    return Err(format!(
                        "Start row {start_row} is greater than end row {end_row}"
                    ));
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
                        let value = self.get_evaluated(col, row)?;
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

    /// Sets all cells in the evaluated table to None.
    fn clear_evaluated_table(&mut self) {
        for col in self.evaluated_table.iter_mut() {
            for row in col.iter_mut() {
                *row = None;
            }
        }
    }

    /// Gets the raw value in the given cell.
    pub fn get_raw(&self, col: usize, row: usize) -> Value {
        if col >= self.raw_table.len() {
            return Value::Null;
        }
        if row >= self.raw_table[col].len() {
            return Value::Null;
        }
        self.raw_table[col][row].clone()
    }

    /// Sets the raw value in the given cell.
    pub fn set_raw(&mut self, col: usize, row: usize, value: Value) {
        if col >= self.raw_table.len() {
            self.raw_table.resize(col + 1, Vec::new());
        }
        if row >= self.raw_table[col].len() {
            self.raw_table[col].resize(row + 1, Value::Null);
        }
        self.raw_table[col][row] = value;

        if col < self.evaluated_table.len() && row < self.evaluated_table[col].len() {
            self.evaluated_table[col][row] = None;
        }
    }

    /// Evaluates the raw value in the given cell and returns the result.
    fn get_evaluated(&mut self, col: usize, row: usize) -> Result<Value, String> {
        if col < self.evaluated_table.len()
            && row < self.evaluated_table[col].len()
            && let Some(value) = self.evaluated_table[col][row].clone()
        {
            return value;
        }

        self.evaluate_cell(col, row);
        self.evaluated_table[col][row].clone().unwrap()
    }

    /// Evaluates the raw values in the given range and returns the results.
    pub fn get_evaluated_range(&mut self, start_col: usize, start_row: usize, end_col: usize, end_row: usize) -> Vec<Vec<Result<Value, String>>> {
        let num_cols = end_col - start_col + 1;
        let num_rows = end_row - start_row + 1;
        let mut results = vec![vec![Ok(Value::Null); num_rows]; num_cols];
        for col in start_col..=end_col {
            for row in start_row..=end_row {
                results[col - start_col][row - start_row] = self.get_evaluated(col, row);
            }
        }

        results
    }

    /// Sets the evaluated value in the given cell.
    fn set_evaluated(&mut self, col: usize, row: usize, value: Result<Value, String>) {
        if col >= self.evaluated_table.len() {
            self.evaluated_table.resize(col + 1, Vec::new());
        }
        if row >= self.evaluated_table[col].len() {
            self.evaluated_table[col].resize(row + 1, None);
        }
        self.evaluated_table[col][row] = Some(value);
    }

    /// Evaluates the raw value in the given cell and sets the result in the evaluated table.
    fn evaluate_cell(&mut self, col: usize, row: usize) {
        let raw_value = self.get_raw(col, row);
        let evaluated_value = self.evaluate_value(&raw_value, col, row);
        self.set_evaluated(col, row, evaluated_value);
    }

    /// Evaluates all cells in the range.
    pub fn evaluate_range(&mut self, start_col: usize, start_row: usize, end_col: usize, end_row: usize) {
        self.clear_evaluated_table();
        for col in start_col..=end_col {
            for row in start_row..=end_row {
                self.evaluate_cell(row, col);
            }
        }
    }

    /// Evaluates all cells in the environment.
    pub fn evaluate_all_cells(&mut self) {
        self.clear_evaluated_table();
        for col in 0..self.raw_table.len() {
            for row in 0..self.raw_table[col].len() {
                self.evaluate_cell(col, row);
            }
        }
    }

    /// Saves the environment to a CSV file.
    pub fn save_to_csv(&mut self, path_to_file: &str) -> Result<(), Box<dyn Error>> {
        self.evaluate_all_cells();

        let mut wtr = csv::Writer::from_path(path_to_file)?;
        if self.evaluated_table.is_empty() {
            wtr.flush()?;
            return Ok(());
        }

        let max_len = self
            .evaluated_table
            .iter()
            .map(|row| row.len())
            .max()
            .unwrap();
        for row in 0..max_len {
            let mut row_values = Vec::new();
            for col in 0..self.evaluated_table.len() {
                let value = self.get_evaluated(col, row);
                let value_str = match value {
                    Ok(value) => value.to_string(),
                    Err(error) => error,
                };
                row_values.push(value_str);
            }
            wtr.write_record(&row_values)?;
        }
        wtr.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_is_subset_of() {
        assert!(ValueType::Int.is_subset_of(&ValueType::Int));
        assert!(ValueType::Int.is_subset_of(&(ValueType::Int | ValueType::Float)));
        assert!(
            ValueType::list_of(ValueType::Int | ValueType::Float).is_subset_of(
                &ValueType::list_of(ValueType::Int | ValueType::Float | ValueType::String)
            )
        );
        assert!(
            (ValueType::Int | ValueType::Float)
                .is_subset_of(&(ValueType::Int | ValueType::Float | ValueType::String))
        );
        assert!(
            (ValueType::list_of(ValueType::list_of(ValueType::Int) | ValueType::List(None))).is_subset_of(
                &(ValueType::list_of(ValueType::list_of(ValueType::Any)) | ValueType::list_of(ValueType::String))
            )
        );
    }
}
