use anyhow::Result;
use std::{
    collections::HashMap,
    fmt::{self},
    net::IpAddr,
    process::id,
};

use crate::{
    error::CompileError,
    parser::{self, ParseToken},
};

const BOILER_FMT: &str = "data $fmt = { b \"%2.4f\n\", b 0 }\n";
const BOILER_POW: &str = "export function $pow(d $x, d $y) d\n";

fn compile(statements: Vec<parser::Statement>) -> Result<String> {
    let mut main_func = Function::new_main();
    let mut functions: Vec<Function> = vec![];
    let mut varcounter = VariableCounter::new();

    for statement in statements {
        match statement {
            parser::Statement::Declaration(declaration) => {

            }
            parser::Statement::Expression(expr) => {
                let (statements, result_id) = compile_expr(expr, &mut varcounter)?;
                main_func.statements.extend_from_slice(&statements);
                main_func.statements.push(Statement::new(identifier, operation));
            }
        }
    }

    let functions_formatted = functions
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    return Ok(format!(
        "{}{}\n{}\n{}",
        BOILER_FMT, BOILER_POW, main_func, functions_formatted
    ));
}

fn compile_expr(expr: Vec<ParseToken>, counter: &mut VariableCounter) -> Result<(Vec<Statement>, String)> {
    let mut compiled: Vec<Statement> = vec![];
    let mut stack: Vec<ParseToken> = vec![];
    for token in expr {
        match token {
            _ if token.is_operator() => {
                if let Some(ParseToken::Identifier(y)) = stack.pop()
                    && let Some(ParseToken::Identifier(x)) = stack.pop()
                {
                    let operation = Operation::Add(x, y);
                    compiled.push(Statement::new(counter.next_temp(), operation));
                } else {
                    return Err(CompileError::OperandError.into());
                }
            }
            _ => return Err(CompileError::InvalidToken(token).into()),
        }
    }
    return Ok((compiled, counter.current_temp()));
}

struct VariableCounter {
    tempcount: i32,
    pairs: HashMap<String, i32>,
}

impl VariableCounter {
    fn new() -> Self {
        VariableCounter{tempcount: 0, pairs: HashMap::new()}
    }

    fn next_var(&mut self, identifier: String) -> String {
        if let Some(count) = self.pairs.get(&identifier) {
            format!("%{}_{}", identifier, count)
        } else {
            self.pairs.insert(identifier.clone(), 0);
            format!("%{}_{}", identifier, 0)
        }
    }

    fn get(&self, identifier: String) -> Result<String> {
        let count = self
            .pairs
            .get(&identifier)
            .ok_or(CompileError::NameError(identifier.clone()))?;
        return Ok(format!("%{}_{}", identifier, count));
    }

    fn next_temp(&mut self) -> String {
        self.tempcount += 1;
        format!("%_{}", self.tempcount)
    }

    fn current_temp(&self) -> String {
        return format!("%_{}", self.tempcount);
    }
}

#[derive(PartialEq, Debug, Clone)]
struct Function {
    export: bool,
    return_type: Type,
    name: String,
    args: Vec<String>,
    statements: Vec<Statement>,
    return_val: String,
}

impl Function {
    fn new(name: String, args: Vec<String>) -> Self {
        Self {
            name,
            args,
            ..Default::default()
        }
    }

    fn new_main() -> Self {
        return Self {
            export: true,
            return_type: Type::Word,
            name: "$main".to_string(),
            args: vec![],
            statements: vec![],
            return_val: "0".to_string(),
        };
    }
}

impl Default for Function {
    fn default() -> Self {
        return Self {
            export: false,
            return_type: Type::Double,
            name: "".to_string(),
            args: vec![],
            statements: vec![],
            return_val: "".to_string(),
        };
    }
}

impl fmt::Display for Function {
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
            "{} function {} ${}({}) {{\n@start\n{}\n\tret {}}}\n",
            export, self.return_type, self.name, args, statements, self.return_val
        )
    }
}

#[derive(PartialEq, Debug, Clone)]
struct Statement {
    identifier: String,
    assign_type: Type,
    operation: Operation,
}

impl Statement {
    fn new(identifier: String, operation: Operation) -> Self {
        Statement {
            identifier,
            assign_type: Type::Double,
            operation,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\t{} ={} {}",
            self.identifier, self.assign_type, self.operation
        )
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

#[derive(PartialEq, Debug, Clone)]
enum Type {
    Word,
    Long,
    Single,
    Double,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.clone() {
                Type::Word => "w",
                Type::Long => "l",
                Type::Single => "s",
                Type::Double => "d",
            }
        )
    }
}
