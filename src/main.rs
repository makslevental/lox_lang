#[macro_use]
extern crate lazy_static;

#[allow(dead_code)]
mod environment;
#[allow(dead_code)]
mod interpreter;
#[allow(dead_code)]
mod lexer;
#[allow(dead_code)]
mod parser;

use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(filepath: &str) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(Path::new(&String::from(filepath)).as_os_str())?;
    run(&contents);
    Ok(())
}

const PROMPT: &str = ">> ";
static HAD_ERROR: bool = false;

fn run_prompt() -> Result<(), Box<dyn Error>> {
    let mut reader = io::BufReader::new(io::stdin());
    let mut line = String::new();

    loop {
        line.clear();
        print!("{}", PROMPT);
        io::stdout().lock().flush()?;

        reader.read_line(&mut line)?;
        run(&line);
    }
}

fn run(source: &String) {
    println!("{}", source)
}

fn error(line: usize, error: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, error, message)
}
