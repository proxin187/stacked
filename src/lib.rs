use std::io::BufWriter;
use std::fs::File;
use std::io::Write;
use std::fmt;

use colored::Colorize;


#[derive(Debug)]
pub enum ExprKind {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, PartialEq)]
pub enum Jump {
    Unconditional,
    Equal,
    NotEqual,
    Greater,
    Lesser,
}

#[derive(Debug, PartialEq)]
pub enum StackOp {
    Push(u32),

    Swap,
    Dump,
    Pop,
    Dup,
    Rot,
    Cmp,
}

#[derive(Debug)]
pub enum MemOp {
    InsertStr(String),
    Store,
    Load,
}

#[derive(Debug)]
pub enum Inst {
    BinaryExpr(ExprKind),
    StackOp(StackOp),
    MemOp(MemOp),
    Jump(Jump, u32),
    Call(u32),

    Label(u32),

    Syscall,
    Return,
    Halt,
}

impl fmt::Display for Inst {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Inst::Jump(jump, addr) => write!(fmt, "{:05} {:013} <{}>", "Jump".yellow(), format!("{:?}", *jump).purple(), format!("{}", *addr).blue())?,
            Inst::BinaryExpr(op) =>   write!(fmt, "{}", format!("{:?}", *op).yellow())?,
            Inst::Label(label) =>     write!(fmt, "{:05} <{}>", "Label".yellow(), format!("{}", *label).blue())?,
            Inst::Call(addr) =>       write!(fmt, "{:05} <{}>", "Call".yellow(), format!("{}", *addr).blue())?,
            Inst::MemOp(op) => {
                match op {
                    MemOp::InsertStr(string) => {
                        write!(fmt, "{:05} {}", "Str".yellow(), format!("{:?}", *string).green())?;
                    },
                    _ => {
                        write!(fmt, "{}", format!("{:?}", *op).yellow())?;
                    },
                }
            },
            Inst::StackOp(op) => {
                match op {
                    StackOp::Push(integer) => write!(fmt, "{:05} ({})", "Push".yellow(), format!("{}", *integer).blue())?,
                    _ => write!(fmt, "{}", format!("{:?}", *op).yellow())?,
                }
            },
            _ => write!(fmt, "{}", format!("{:?}", *self).yellow())?,
        }

        Ok(())
    }
}

pub struct CodeGen {
    instructions: Vec<Inst>,
    writer: BufWriter<File>,
}

impl CodeGen {
    pub fn new(file: &str) -> Result<CodeGen, Box<dyn std::error::Error>> {
        Ok(CodeGen {
            instructions: Vec::new(),
            writer: BufWriter::new(File::create(file)?),
        })
    }

    pub fn append(&mut self, inst: Inst) {
        self.instructions.push(inst);
    }

    fn output_int(&self, integer: u32) -> [u8; 4] {
        if cfg!(target_endian = "big") {
            integer.to_be_bytes()
        } else {
            integer.to_le_bytes()
        }
    }

    pub fn output(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for inst in &self.instructions {
            match inst {
                Inst::Label(ident) => {
                    self.writer.write_all(&[0x4C])?;

                    self.writer.write_all(&self.output_int(*ident))?;
                },
                Inst::Call(addr) => {
                    self.writer.write_all(&[0x2F])?;

                    self.writer.write_all(&self.output_int(*addr))?;
                },
                Inst::Jump(condition, addr) => {
                    self.writer.write_all(&[
                        match condition {
                            Jump::Unconditional => 0x6A,
                            Jump::Equal =>         0x6B,
                            Jump::NotEqual =>      0x6E,
                            Jump::Greater =>       0x6C,
                            Jump::Lesser =>        0x6D,
                        }
                    ])?;

                    self.writer.write_all(&self.output_int(*addr))?;
                },
                Inst::StackOp(op) => {
                    match op {
                        StackOp::Push(integer) => {
                            self.writer.write_all(&[0x01])?;

                            self.writer.write_all(&self.output_int(*integer))?;
                        },
                        StackOp::Pop => {
                            self.writer.write_all(&[0x02])?;
                        },
                        StackOp::Dup => {
                            self.writer.write_all(&[0x05])?;
                        },
                        StackOp::Swap => {
                            self.writer.write_all(&[0x06])?;
                        },
                        StackOp::Rot => {
                            self.writer.write_all(&[0x07])?;
                        },
                        StackOp::Dump => {
                            self.writer.write_all(&[0x03])?;
                        },
                        StackOp::Cmp => {
                            self.writer.write_all(&[0x43])?;
                        },
                    }
                },
                Inst::MemOp(op) => {
                    match op {
                        MemOp::InsertStr(string) => {
                            self.writer.write_all(&[0x8C])?;

                            self.writer.write_all(string.as_bytes())?;
                        },
                        MemOp::Load => {
                            self.writer.write_all(&[0x8A])?;
                        },
                        MemOp::Store => {
                            self.writer.write_all(&[0x8B])?;
                        },
                    }
                },
                Inst::BinaryExpr(kind) => {
                    self.writer.write_all(&[
                        match kind {
                            ExprKind::Add => 0x28,
                            ExprKind::Sub => 0x29,
                            ExprKind::Mul => 0x2A,
                            ExprKind::Div => 0x2B,
                        }
                    ])?;
                },
                Inst::Syscall => {
                    self.writer.write_all(&[0x53])?;
                },
                Inst::Return => {
                    self.writer.write_all(&[0x0D])?;
                },
                Inst::Halt => {
                    self.writer.write_all(&[0x04])?;
                },
            }
        }

        self.writer.flush()?;

        Ok(())
    }
}


