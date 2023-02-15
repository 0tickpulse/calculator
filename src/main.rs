use std::io::Write;

pub mod scanner;
pub mod parser;
mod errors;
mod interpreter;

fn main() {
    let debug = std::env::args().any(|arg| arg == "--debug");
    repl(debug);
}

fn calculate(source: String) -> Result<f64, errors::CalculatorError> {
    let mut scanner = scanner::Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = parser::Parser::new(tokens);
    let expr = parser.parse()?;
    let mut interpreter = interpreter::Interpreter::new();
    let result = interpreter.interpret(&*expr)?;
    Ok(result)
}

fn calculate_with_debug(source: String) -> Result<f64, errors::CalculatorError> {
    let mut scanner = scanner::Scanner::new(source);
    let tokens = scanner.scan_tokens();
    println!("Tokens: {:?}", tokens);
    let mut parser = parser::Parser::new(tokens);
    let expr = parser.parse()?;
    println!("AST: {:?}", expr);
    let mut interpreter = interpreter::Interpreter::new();
    let result = interpreter.interpret(&*expr)?;
    Ok(result)
}

fn repl(debug: bool) {
    // prompt console
    let debug_text = if debug { " (debug mode)" } else { "" };
    println!("Welcome to the calculator!{debug_text}");
    println!("Enter an expression to evaluate it, or 'exit' to quit.");
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "exit" {
            break;
        }
        let result = if debug {
            calculate_with_debug(input.to_string())
        } else {
            calculate(input.to_string())
        };
        match result {
            Ok(result) => println!("Result: {}", result),
            Err(error) => println!("Error: {}", error),
        }
    }
}
