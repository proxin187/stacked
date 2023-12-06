mod syscall;
mod parser;
mod exec;
mod log;

use parser::Parser;
use exec::Machine;
use stacked::*;

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
            // TODO: remove for production
            let mut codegen = match CodeGen::new(&file) {
                Ok(codegen) => codegen,
                Err(err) => {
                    log::error(&format!("failed to initialize codegen: {}", err.to_string()));
                    process::exit(1);
                },
            };

            /*
            // poke string into memory
            for (index, character) in "Skibidi toilet only in ohio ngl\n\0".chars().enumerate() {
                codegen.append(Inst::StackOp(StackOp::Push(character as u32)));
                codegen.append(Inst::StackOp(StackOp::Push(index as u32)));
                codegen.append(Inst::MemOp(MemOp::Store));
            }
            */

            codegen.append(Inst::StackOp(StackOp::Push(0)));
            codegen.append(Inst::MemOp(MemOp::InsertStr(String::from("Skibidi toilet, Skibidi Skibidi toilet\n\0"))));

            /* PRINT
            // count
            codegen.append(Inst::StackOp(StackOp::Push(14)));
            // buf (addr)
            codegen.append(Inst::StackOp(StackOp::Push(0)));
            // fd
            codegen.append(Inst::StackOp(StackOp::Push(1)));
            // syscall (write)
            codegen.append(Inst::StackOp(StackOp::Push(1)));
            codegen.append(Inst::Syscall);
            */

            // strlen(str)
            codegen.append(Inst::StackOp(StackOp::Push(0)));
            codegen.append(Inst::Call(0));

            codegen.append(Inst::StackOp(StackOp::Dump));

            codegen.append(Inst::Halt);

            // strlen
            codegen.append(Inst::Label(0));

            codegen.append(Inst::StackOp(StackOp::Push(0)));

            // while (str[idx] != '\0') {
            codegen.append(Inst::Label(1));

            // idx = idx + 1;
            codegen.append(Inst::StackOp(StackOp::Push(1)));
            codegen.append(Inst::BinaryExpr(ExprKind::Add));

            // str[idx]
            codegen.append(Inst::StackOp(StackOp::Swap));
            codegen.append(Inst::StackOp(StackOp::Dup));
            codegen.append(Inst::StackOp(StackOp::Rot));
            codegen.append(Inst::StackOp(StackOp::Dup));
            codegen.append(Inst::StackOp(StackOp::Rot));
            codegen.append(Inst::StackOp(StackOp::Swap));

            // addr + idx
            codegen.append(Inst::BinaryExpr(ExprKind::Add));

            // mem[addr + idx]
            codegen.append(Inst::MemOp(MemOp::Load));


            // str[idx] != '\0'
            codegen.append(Inst::StackOp(StackOp::Push(0)));
            codegen.append(Inst::StackOp(StackOp::Cmp));

            // }
            codegen.append(Inst::Jump(Jump::Equal, 2));
            codegen.append(Inst::Jump(Jump::Unconditional, 1));

            codegen.append(Inst::Label(2));
            codegen.append(Inst::Return);

            if let Err(err) = codegen.output() {
                log::error(&format!("failed to output byte code: {}", err.to_string()));
                process::exit(1);
            }

            let mut parser = match Parser::new(&file) {
                Ok(parser) => parser,
                Err(err) => {
                    log::error(&format!("failed to initialize parser: {}", err.to_string()));
                    process::exit(1);
                },
            };

            if let Commands::Exec { .. } = args.command {
                let instructions = match parser.parse(false) {
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
                match parser.parse(true) {
                    Err(err) => {
                        log::error(&format!("failed to parse: {}", err.to_string()));
                        process::exit(1);
                    },
                    Ok(_) => (),
                }
            }

        },
    }

}

