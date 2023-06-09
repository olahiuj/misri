mod env;
mod exec;
mod instr;
mod lexer;
mod parser;
mod value;

use clap::{arg, Command};
use parser::Parser;
use std::{fs, io};

use crate::exec::Interpreter;

fn main() {
    let matches = Command::new("misri")
        .version("0.1.0")
        .author("jjppp <jpwang@smail.nju.edu.cn>")
        .about("Yet another interpreter for NJU irsim")
        .arg(arg!(-f --file <FILE> "ir file"))
        .get_matches();

    let file = match matches.get_one::<String>("file") {
        Some(file) => file,
        None => panic!("arg error"),
    };

    let cont = fs::read_to_string(file).expect("file error");
    let mut parser = Parser::from(cont.as_str());
    let program = parser.parse();
    let mut interpreter = Interpreter::new(program, io::stdin(), io::stdout());
    let instr_cnt = interpreter.exec();
    eprintln!("instrCnt: {instr_cnt}")
}
