mod scanner;
mod token;
use scanner::Scanner;
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
    println!("{}", source);
    match Scanner::scan(source) {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token)
            }
        }
        Err(errors) => {
            for error in errors {
                report_error(&error)
            }
        }
    }
}

fn report_error(error: &dyn std::error::Error) {
    println!("Error: {}", error)
}
