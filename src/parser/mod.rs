use std::io::{BufReader, Read};
use std::collections::HashMap;
use std::fs::File;
use std::mem;

use crate::{ExprKind, Jump, StackOp, Inst, MemOp};


pub struct Parser {
    reader: BufReader<File>,
    pub labels: HashMap<u32, u32>,
}

impl Parser {
    pub fn new(file: &str) -> Result<Parser, Box<dyn std::error::Error>> {
        Ok(Parser {
            reader: BufReader::new(File::open(file)?),
            labels: HashMap::new(),
        })
    }

    fn from_bytes(&self, bytes: [u8; 4]) -> u32 {
        if cfg!(target_endian = "big") {
            u32::from_be_bytes(bytes)
        } else {
            u32::from_le_bytes(bytes)
        }
    }

    pub fn parse(&mut self, disassemble: bool) -> Result<Vec<Inst>, Box<dyn std::error::Error>> {
        let mut instructions: Vec<Inst> = Vec::new();
        let mut disassembly = String::new();
        let mut ip = 0;

        loop {
            let mut buffer = [0u8; mem::size_of::<u8>()];

            match self.reader.read_exact(&mut buffer) {
                Err(_) => break,
                _ => {},
            }

            if disassemble {
                if !instructions.is_empty() {
                    println!("{:040}     {}", disassembly, instructions[instructions.len() - 1]);
                }

                disassembly = format!("{:#07x}: {:#04x} ", ip, buffer[0]);
            }

            match buffer[0] {
                0x4C | 0x01 | 0x6A | 0x6B | 0x6C | 0x6D | 0x2F => {
                    let mut value = [0u8; mem::size_of::<u32>()];

                    match self.reader.read_exact(&mut value) {
                        Err(_) => break,
                        _ => {},
                    }

                    if disassemble {
                        for byte in value {
                            disassembly += &format!("{:#04x} ", byte);
                        }
                    }

                    match buffer[0] {
                        0x4C => {
                            self.labels.insert(self.from_bytes(value), instructions.len() as u32);
                            instructions.push(Inst::Label(self.from_bytes(value)));
                        },

                        0x01 => { instructions.push(Inst::StackOp(StackOp::Push(self.from_bytes(value)))); },

                        0x2F => { instructions.push(Inst::Call(self.from_bytes(value))); },

                        0x6A => { instructions.push(Inst::Jump(Jump::Unconditional, self.from_bytes(value))); },
                        0x6B => { instructions.push(Inst::Jump(Jump::Equal, self.from_bytes(value))); },
                        0x6E => { instructions.push(Inst::Jump(Jump::NotEqual, self.from_bytes(value))); },
                        0x6C => { instructions.push(Inst::Jump(Jump::Greater, self.from_bytes(value))); },
                        0x6D => { instructions.push(Inst::Jump(Jump::Lesser, self.from_bytes(value))); },

                        _ => {},
                    }
                },
                0x8C => {
                    let mut string = String::new();

                    loop {
                        let mut character = [0u8; mem::size_of::<u8>()];

                        match self.reader.read_exact(&mut character) {
                            Err(_) => break,
                            _ => {},
                        }

                        string.push(character[0] as char);

                        if character[0] as char == '\0' {
                            break;
                        }
                    }

                    instructions.push(Inst::MemOp(MemOp::InsertStr(string)));
                },
                0x8A => { instructions.push(Inst::MemOp(MemOp::Load)); },
                0x8B => { instructions.push(Inst::MemOp(MemOp::Store)); },

                0x02 => { instructions.push(Inst::StackOp(StackOp::Pop)); },
                0x05 => { instructions.push(Inst::StackOp(StackOp::Dup)); },
                0x06 => { instructions.push(Inst::StackOp(StackOp::Swap)); },
                0x07 => { instructions.push(Inst::StackOp(StackOp::Rot)); },
                0x03 => { instructions.push(Inst::StackOp(StackOp::Dump)); },
                0x43 => { instructions.push(Inst::StackOp(StackOp::Cmp)); },

                0x28 => { instructions.push(Inst::BinaryExpr(ExprKind::Add)); },
                0x29 => { instructions.push(Inst::BinaryExpr(ExprKind::Sub)); },
                0x2A => { instructions.push(Inst::BinaryExpr(ExprKind::Mul)); },
                0x2B => { instructions.push(Inst::BinaryExpr(ExprKind::Div)); },

                0x53 => { instructions.push(Inst::Syscall); },
                0x0D => { instructions.push(Inst::Return); },
                0x04 => { instructions.push(Inst::Halt); },
                _ => {},
            }

            ip += 1;
        }

        Ok(instructions)
    }
}


