use crate::environment::{Value, ValueType};
use std::collections::HashMap;

pub struct Function {
    name: String,
    argument_types: Vec<ValueType>,
    function: fn(&[Value]) -> Result<Value, String>,
}

impl Function {
    pub fn call(&self, arguments: &[Value]) -> Result<Value, String> {
        if arguments.len() != self.argument_types.len() {
            return Err(format!(
                "Function `{}` expects {} arguments, but got {}",
                self.name,
                self.argument_types.len(),
                arguments.len()
            ));
        }
        for (i, (argument, argument_type)) in
            arguments.iter().zip(self.argument_types.iter()).enumerate()
        {
            if !argument.type_().is_subset_of(argument_type) {
                return Err(format!(
                    "Function `{}` expects argument {} to be of type `{}`, but got a `{}` instead",
                    self.name,
                    i + 1,
                    argument_type,
                    argument.type_()
                ));
            }
        }

        (self.function)(arguments)
    }
}

pub fn builtin_functions() -> HashMap<String, Function> {
    let mut functions = HashMap::new();

    functions.insert(
        "sum".to_string(),
        Function {
            name: "sum".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::Int | ValueType::Float)],
            function: |values| {
                let Value::List(values) = &values[0] else {
                    unreachable!()
                };
                let sum = values
                    .iter()
                    .map(|v| match v {
                        Value::Int(value) => *value as f64,
                        Value::Float(value) => *value,
                        _ => unreachable!(),
                    })
                    .sum::<f64>();
                let result = if sum.fract() == 0.0 {
                    Value::Int(sum as i64)
                } else {
                    Value::Float(sum)
                };

                Ok(result)
            },
        },
    );
    functions.insert(
        "min".to_string(),
        Function {
            name: "min".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::Int | ValueType::Float)],
            function: |values| {
                let Value::List(values) = &values[0] else {
                    unreachable!()
                };
                let values = values
                    .iter()
                    .map(|v| match v {
                        Value::Int(value) => *value as f64,
                        Value::Float(value) => *value,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<f64>>();

                let min = *values.iter().min_by(|a, b| f64::total_cmp(a, b)).ok_or("Internal error in `min` function")?;
                let result = if min.fract() == 0.0 {
                    Value::Int(min as i64)
                } else {
                    Value::Float(min)
                };
                Ok(result)
            },
        },
    );
    functions.insert(
        "max".to_string(),
        Function {
            name: "max".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::Int | ValueType::Float)],
            function: |values| {
                let Value::List(values) = &values[0] else {
                    unreachable!()
                };
                let values = values
                    .iter()
                    .map(|v| match v {
                        Value::Int(value) => *value as f64,
                        Value::Float(value) => *value,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<f64>>();

                let max = *values.iter().max_by(|a, b| f64::total_cmp(a, b)).ok_or("Internal error in `max` function")?;
                let result = if max.fract() == 0.0 {
                    Value::Int(max as i64)
                } else {
                    Value::Float(max)
                };
                Ok(result)
            },
        },
    );
    functions.insert(
        "mean".to_string(),
        Function {
            name: "mean".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::Int | ValueType::Float)],
            function: |values| {
                let Value::List(values) = &values[0] else {
                    unreachable!()
                };
                let values = values
                    .iter()
                    .map(|v| match v {
                        Value::Int(value) => *value as f64,
                        Value::Float(value) => *value,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<f64>>();

                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let result = if mean.fract() == 0.0 {
                    Value::Int(mean as i64)
                } else {
                    Value::Float(mean)
                };
                Ok(result)
            },
        },
    );
    functions.insert(
        "count".to_string(),
        Function {
            name: "count".to_string(),
            argument_types: vec![ValueType::list_of(
                ValueType::Bool | ValueType::Int | ValueType::Float | ValueType::String,
            )],
            function: |values| {
                let Value::List(values) = &values[0] else {
                    unreachable!()
                };
                let count = values.len();
                Ok(Value::Int(count as i64))
            },
        },
    );

    functions
}
