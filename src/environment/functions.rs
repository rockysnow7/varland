use crate::environment::{Value, ValueType};
use std::collections::HashMap;

pub struct Function {
    name: String,
    description: String,
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
            description: "Takes a list of numbers and returns the sum of the numbers".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::Int | ValueType::Float)],
            function: |args| {
                let Value::List(values) = &args[0] else {
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
            description: "Takes a list of numbers and returns the smallest number".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::Int | ValueType::Float)],
            function: |args| {
                let Value::List(values) = &args[0] else {
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
            description: "Takes a list of numbers and returns the largest number".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::Int | ValueType::Float)],
            function: |args| {
                let Value::List(values) = &args[0] else {
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
            description: "Takes a list of numbers and returns the mean of the numbers".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::Int | ValueType::Float)],
            function: |args| {
                let Value::List(values) = &args[0] else {
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
        "len".to_string(),
        Function {
            name: "len".to_string(),
            description: "Takes a list or string and returns its length".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::Any) | ValueType::String],
            function: |args| {
                match &args[0] {
                    Value::List(values) => Ok(Value::Int(values.len() as i64)),
                    Value::String(value) => Ok(Value::Int(value.len() as i64)),
                    _ => unreachable!(),
                }
            },
        },
    );

    functions.insert(
        "join".to_string(),
        Function {
            name: "join".to_string(),
            description: "Takes either a list of lists and joins them into a single list, or a list of strings and joins them into a single string".to_string(),
            argument_types: vec![ValueType::list_of(ValueType::list_of(ValueType::Any)) | ValueType::list_of(ValueType::String)],
            function: |args| {
                if args[0].type_().is_subset_of(&ValueType::list_of(ValueType::list_of(ValueType::Any))) {
                    let Value::List(lists) = &args[0] else { unreachable!() };
                    let mut result = Vec::new();
                    for list in lists {
                        let Value::List(values) = list else { unreachable!() };
                        result.extend(values.iter().cloned());
                    }
                    Ok(Value::List(result))
                } else {
                    let Value::List(strings) = &args[0] else { unreachable!() };
                    let result = strings.iter().map(|s| match s {
                        Value::String(value) => value.clone(),
                        _ => unreachable!(),
                    }).collect::<String>();
                    Ok(Value::String(result))
                }
            },
        },
    );

    functions
}
