mod as_generator;
mod as_intrinsics;
mod assembler;
mod compiler;
mod interpreter;
mod ir_generator;
mod parser;
mod tokenizer;
mod typechecker;
mod util;

use base64::prelude::*;
use clap::Parser;
use serde_json::{json, Value};
use std::io::prelude::*;
use std::net::TcpListener;
use std::{
    fs,
    io::{self, Read},
};

#[derive(clap::ValueEnum, Clone, Debug)]
enum Command {
    Compile,
    Serve,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum)]
    command: Command,

    #[arg(short, long)]
    input_file: Option<String>,

    #[arg(long)]
    output_file: Option<String>,

    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value_t = 3000)]
    port: u32,
}

fn read_source(input_file: Option<String>) -> String {
    match input_file {
        Some(input_file) => fs::read_to_string(&input_file)
            .unwrap_or_else(|_| panic!("Unable to read input file {input_file}")),
        None => {
            let mut input = Vec::new();
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            handle.read_to_end(&mut input).unwrap();
            String::from_utf8(input).unwrap()
        }
    }
}

fn run_server(host: String, port: u32) {
    let address = format!("{host}:{port}");
    println!("Running server at {address}");
    let listener = TcpListener::bind(address).unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let request: Value = serde_json::from_reader(&stream).unwrap();

        let mut result = json!({});
        if request["command"] == "compile" {
            let source_code = &request["code"].as_str().unwrap();
            let output = std::panic::catch_unwind(|| compiler::compile(source_code.to_string()));

            if let Ok(binary) = output {
                let base64 = BASE64_STANDARD.encode(binary);
                result["program"] = Value::String(base64);
            } else {
                result["error"] = Value::String(String::from("Compile failed"));
            }
        } else if request["command"] != "ping" {
            panic!()
        }

        stream.write_all(result.to_string().as_bytes()).unwrap();
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Compile => {
            let source_code = read_source(args.input_file);
            compiler::compile(source_code);
        }
        Command::Serve => run_server(args.host, args.port),
    }
}
