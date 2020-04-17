#![warn(clippy::all)]
#![warn(clippy::pedantic)]

mod environment;
mod expression;
mod interpreter;
mod parser;
mod scanner;
mod statement;
mod token;
mod value;

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use std::io::Write;

struct Lox<'a> {
    interpreter: Interpreter<'a>,
}

impl<'a> Lox<'a> {
    fn run_file<'b: 'a>(&'b mut self, path: &str) {
        println!("{}", path);
        let contents =
            std::fs::read_to_string(path).expect("Something went wrong when reading file");
        self.run(&contents);
    }

    fn run_prompt<'b: 'a>(&'b mut self) {
        let mut input = String::new();

        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            std::io::stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
            self.run(&input);
            input.clear();
        }
    }

    fn run<'b: 'a>(&'b mut self, source: &str) {
        let result = Scanner::scan(source)
            .and_then(|tokens| Parser::parse(&tokens))
            .and_then(|statements| {
                self.interpreter
                    .interpret(&statements)
                    .map_err(|error| vec![error])
            });
        match result {
            Ok(_) => {}
            Err(strings) => {
                for string in strings {
                    println!("Error: {}", string)
                }
            }
        }
    }
}

fn main() {
    let mut lox = Lox {
        interpreter: Interpreter::new(),
    };

    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => lox.run_prompt(),
        2 => lox.run_file(&args[1]),
        _ => {
            println!("usage: rlox [script]");
            std::process::exit(64);
        }
    }
}
