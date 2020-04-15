mod scanner;
mod token;
mod parser;
mod expression;
mod interpreter;
mod value;

use scanner::Scanner;
use parser::Parser;
use interpreter::Interpreter;

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[0]),
        _ => {
            println!("usage: rlox [script]");
            std::process::exit(64);
        }
    }
}

fn run_file(path: &str) {
    let contents = std::fs::read_to_string(path).expect("Something went wrong when reading file");
    run(&contents);
}

fn run_prompt() {
    let mut input = String::new();

    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        run(&input);
        input.clear();
    }
}

fn run(source: &str) {
    let expr = Scanner::scan(source).and_then(|tokens| {
        Parser::parse(&tokens)
    }).and_then(|expr| {
        let interpreter = Interpreter {};
        interpreter.interpret(expr).map_err(|error| { vec![error] })
    });
    match expr {
        Ok(expr) => println!("{}", expr),
        Err(strings) => {
            for string in strings {
                println!("Error: {}", string)
            }
        }
    }
}

