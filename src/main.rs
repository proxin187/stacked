mod disassemble;
mod syscall;
mod parser;
mod exec;
mod log;

use parser::Parser;
use exec::Machine;
use lib_stacked::*;

use clap::{Parser as ClapParser, Subcommand};

use std::process;

#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, short, action)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Exec { file: String },
    Disassemble { file: String },
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Exec { file } | Commands::Disassemble { file } => {
            let mut parser = match Parser::new(&file) {
                Ok(parser) => parser,
                Err(err) => {
                    log::error(&format!("failed to initialize parser: {}", err.to_string()));
                    process::exit(1);
                },
            };

            if let Commands::Exec { .. } = args.command {
                let instructions = match parser.parse() {
                    Ok(instructions) => instructions,
                    Err(err) => {
                        log::error(&format!("failed to parse: {}", err.to_string()));
                        process::exit(1);
                    },
                };

                let mut vm = Machine::new(args.debug);

                match vm.exec(instructions, parser.labels) {
                    Err(exec::ErrorKind::StackUnderflow) => {
                        log::error("stackunderflow");
                        process::exit(1);
                    },
                    Err(exec::ErrorKind::OutOfBounds) => {
                        log::error("out of bounds");
                        process::exit(1);
                    },
                    Err(exec::ErrorKind::UnknownLabel(addr)) => {
                        log::error(&format!("unknown label `{addr}`"));
                        process::exit(1);
                    },
                    Err(exec::ErrorKind::Syscall(err)) => {
                        log::error(&format!("{err}"));
                        process::exit(1);
                    },
                    Err(exec::ErrorKind::UnknownSyscall) => {
                        log::error("unknown syscall");
                        process::exit(1);
                    },
                    Ok(_) => (),
                }
            } else {
                let instructions = match parser.parse() {
                    Ok(instructions) => instructions,
                    Err(err) => {
                        log::error(&format!("failed to parse: {}", err.to_string()));
                        process::exit(1);
                    },
                };

                disassemble::disassemble(instructions);
            }

        },
    }

}

