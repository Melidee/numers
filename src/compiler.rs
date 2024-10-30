use std::fmt;
use anyhow::Result;

use crate::parser::Statement;

fn compile(statements: Vec<Statement>) -> Result<String> {
    todo!()
}

#[derive(PartialEq, Debug, Clone)]
struct QbeFunction {
    export: bool,
    return_type: String,
    name: String,
    args: Vec<String>,
    statements: Vec<QbeStatement>,
    return_val: String,
}

impl fmt::Display for QbeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let export = if self.export { "export " } else { "" };
        let args = self
            .args
            .iter()
            .map(|arg| format!("d %{arg}"))
            .collect::<Vec<String>>()
            .join(", ");
        let statements = self
            .statements
            .iter()
            .map(|stmt| stmt.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        write!(
            f,
            "{} function {} ${}({}) {{\n@start\n{}\n\tret {}}}",
            export, self.return_type, self.name, args, statements, self.return_val
        )
    }
}

#[derive(PartialEq, Debug, Clone)]
struct QbeStatement {
    var: String,
    assign_type: String,
    operation: Operation,
}

impl fmt::Display for QbeStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\t{} ={} {}", self.var, self.assign_type, self.operation)
    }
}

#[derive(PartialEq, Debug, Clone)]
enum Operation {
    Add(String, String),
    Sub(String, String),
    Div(String, String),
    Mul(String, String),
    Pow(String, String),
    Call(String),
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.clone() {
                Operation::Add(x, y) => format!("add %{x} %{y}"),
                Operation::Sub(x, y) => format!("sub %{x} %{y}"),
                Operation::Mul(x, y) => format!("mul %{x} %{y}"),
                Operation::Div(x, y) => format!("div %{x} %{y}"),
                Operation::Pow(x, y) => format!("call $pow(d {x}, d {y})"),
                Operation::Call(func) => format!("call {func}"),
            }
        )
    }
}
