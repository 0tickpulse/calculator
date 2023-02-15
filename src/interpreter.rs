use crate::{
    errors::{CalculatorError, CalculatorErrorType},
    parser::expressions::*,
    scanner::TokenType,
};
use std::{collections::HashMap, f64::consts};

pub struct Interpreter {
    pub variables: HashMap<String, f64>,
    pub single_functions: HashMap<String, fn(f64) -> f64>,
    pub double_functions: HashMap<String, fn(f64, f64) -> f64>,
}

const PHI: f64 = 1.618033988749895;

impl Visitor for Interpreter {
    fn visit_binary_expr(&mut self, expr: &Binary) -> Result<f64, CalculatorError> {
        let left = self.interpret(&*expr.left)?;
        let right = self.interpret(&*expr.right)?;

        Ok(match expr.operator.kind {
            TokenType::Plus => left + right,
            TokenType::Minus => left - right,
            TokenType::Star => left * right,
            TokenType::Slash => left / right,
            TokenType::Caret => left.powf(right),
            _ => todo!(),
        })
    }

    fn visit_grouping_expr(&mut self, expr: &Grouping) -> Result<f64, CalculatorError> {
        self.interpret(&*expr.expression)
    }

    fn visit_literal_expr(&mut self, expr: &Literal) -> Result<f64, CalculatorError> {
        Ok(expr.value.literal.unwrap_or(0.0))
    }

    fn visit_unary_expr(&mut self, expr: &Unary) -> Result<f64, CalculatorError> {
        let right = self.interpret(&*expr.right)?;

        match expr.operator.kind {
            TokenType::Minus => Ok(-right),
            TokenType::Plus => Ok(right),
            _ => todo!(),
        }
    }

    fn visit_call_expr(&mut self, expr: &Call) -> Result<f64, CalculatorError> {
        let mut arguments = expr
            .arguments
            .iter()
            .map(|arg| self.interpret(&**arg))
            .collect::<Vec<_>>();

        let name = &expr.callee.lexeme;

        if let Some(function) = self.single_functions.get(name) {
            if arguments.len() != 1 {
                return Err(CalculatorError {
                    error: CalculatorErrorType::FunctionArityMismatch(
                        name.to_string(),
                        arguments.len(),
                        1,
                    ),
                    token: None,
                });
            }
            let argument = arguments.remove(0)?;
            Ok(function(argument))
        } else if let Some(function) = self.double_functions.get(name) {
            if arguments.len() != 2 {
                return Err(CalculatorError {
                    error: CalculatorErrorType::FunctionArityMismatch(
                        name.to_string(),
                        arguments.len(),
                        2,
                    ),
                    token: None,
                });
            }
            let left = arguments.remove(0)?;
            let right = arguments.remove(0)?;
            Ok(function(left, right))
        } else {
            Err(CalculatorError {
                error: CalculatorErrorType::UndefinedVariableOrFunction(name.to_string()),
                token: None,
            })
        }
    }

    fn visit_variable_expr(&mut self, expr: &Variable) -> Result<f64, CalculatorError> {
        let name = &expr.name.lexeme;
        if let Some(value) = self.variables.get(name) {
            Ok(*value)
        } else {
            Err(CalculatorError {
                error: CalculatorErrorType::UndefinedVariableOrFunction(name.to_string()),
                token: None,
            })
        }
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut interpreter = Interpreter {
            variables: HashMap::new(),
            single_functions: HashMap::new(),
            double_functions: HashMap::new(),
        };
        interpreter.add_things();
        interpreter
    }

    fn add_things(&mut self) {
        self.add_variable("pi", consts::PI)
            .add_variable("e", consts::E)
            .add_variable("tau", consts::TAU)
            .add_variable("phi", PHI)
            .add_single_function("sin", f64::sin)
            .add_single_function("cos", f64::cos)
            .add_single_function("tan", f64::tan)
            .add_single_function("asin", f64::asin)
            .add_single_function("acos", f64::acos)
            .add_single_function("atan", f64::atan)
            .add_single_function("sinh", f64::sinh)
            .add_single_function("cosh", f64::cosh)
            .add_single_function("tanh", f64::tanh)
            .add_single_function("asinh", f64::asinh)
            .add_single_function("acosh", f64::acosh)
            .add_single_function("atanh", f64::atanh)
            .add_single_function("sqrt", f64::sqrt)
            .add_single_function("cbrt", f64::cbrt)
            .add_single_function("exp", f64::exp)
            .add_single_function("exp2", f64::exp2)
            .add_single_function("ln", f64::ln)
            .add_single_function("log2", f64::log2)
            .add_single_function("log10", f64::log10)
            .add_single_function("abs", f64::abs)
            .add_single_function("signum", f64::signum)
            .add_single_function("floor", f64::floor)
            .add_single_function("ceil", f64::ceil)
            .add_single_function("round", f64::round)
            .add_single_function("trunc", f64::trunc)
            .add_double_function("pow", f64::powf)
            .add_double_function("atan2", f64::atan2)
            .add_double_function("hypot", f64::hypot)
            .add_double_function("max", f64::max)
            .add_double_function("min", f64::min)
            .add_double_function("remainder", f64::rem_euclid)
            .add_double_function("fmod", f64::rem_euclid);
    }

    /// Simple utility function to add a variable to the interpreter
    /// Returns self for chaining.
    fn add_variable(&mut self, name: &str, value: f64) -> &mut Interpreter {
        self.variables.insert(name.to_string(), value);
        self
    }

    /// Simple utility function to add a single argument function to the interpreter
    /// Returns self for chaining.
    fn add_single_function(&mut self, name: &str, function: fn(f64) -> f64) -> &mut Interpreter {
        self.single_functions.insert(name.to_string(), function);
        self
    }

    /// Simple utility function to add a double argument function to the interpreter
    /// Returns self for chaining.
    fn add_double_function(
        &mut self,
        name: &str,
        function: fn(f64, f64) -> f64,
    ) -> &mut Interpreter {
        self.double_functions.insert(name.to_string(), function);
        self
    }

    pub fn interpret(&mut self, expr: &dyn Expression) -> Result<f64, CalculatorError> {
        expr.accept(self)
    }
}
