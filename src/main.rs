use varland::environment::{Environment, Value};

fn main() {
    let mut environment = Environment::new();
    environment.set_value_raw(0, 0, Value::Int(1));
    environment.set_value_raw(0, 1, Value::Float(2.1));
    environment.set_value_raw(1, 0, Value::CloneCell { col: 0, row: 0 });
    environment.set_value_raw(2, 0, Value::FunctionCall {
        function_name: "mean".to_string(),
        arguments: vec![Value::CloneCellRange {
            start_col: 0,
            start_row: 0,
            end_col: 1,
            end_row: 2,
        }],
    });
    environment.to_csv("test.csv").unwrap();
}
