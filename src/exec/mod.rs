use crate::{Inst, ExprKind, Jump, StackOp, MemOp, log, syscall::{self, Syscall}};

use std::collections::HashMap;
use std::fmt;
use std::io;

#[derive(Clone, Copy, Debug)]
pub enum Value {
    Int(u32),
}

impl Value {
    pub fn as_int(&self) -> u32 {
        match self {
            Value::Int(integer) => *integer,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(integer) => {
                write!(f, "{integer}")
            },
        }
    }
}

pub enum ErrorKind {
    UnknownLabel(u32),
    Syscall(String),

    UnknownSyscall,
    StackUnderflow,
    OutOfBounds,
}

pub struct Machine {
    ret_stack: Vec<u32>,
    stack: Vec<Value>,
    memory: [Value; 10000],
    debug: bool,
}

impl Machine {
    pub fn new(debug: bool) -> Machine {
        Machine {
            ret_stack: Vec::new(),
            stack: Vec::new(),
            memory: [Value::Int(0); 10000],
            debug,
        }
    }

    fn pop(&mut self) -> Result<Value, ErrorKind> {
        if self.stack.len() > 0 {
            Ok(self.stack.pop().unwrap())
        } else {
            Err(ErrorKind::StackUnderflow)
        }
    }

    fn jump(&mut self, labels: &HashMap<u32, u32>, ip: &mut u32, addr: u32) -> Result<(), ErrorKind> {
        if let Some(addr) = labels.get(&addr) {
            *ip = *addr;
        } else {
            return Err(ErrorKind::UnknownLabel(addr));
        }

        Ok(())
    }

    fn strlen(&self, ptr: usize) -> usize {
        self.memory[ptr..]
            .iter()
            .map(|value| if value.as_int() == 0 { return })
            .count()
    }

    fn to_string(&self, bytes: &[Value]) -> String {
        bytes
            .iter()
            .map(|byte| byte.as_int().clamp(0, 255) as u8 as char)
            .collect::<String>()
    }

    fn bound_check(&self, addr: u32) -> Result<(), ErrorKind> {
        if (addr as usize) < self.memory.len() {
            Ok(())
        } else {
            Err(ErrorKind::OutOfBounds)
        }
    }

    pub fn exec(&mut self, instructions: Vec<Inst>, labels: HashMap<u32, u32>) -> Result<(), ErrorKind> {
        let mut ip = 0;

        while ip < instructions.len() as u32 {
            if self.debug {
                log::info(&format!("Inst: {:?}", instructions[ip as usize]));
            }

            match &instructions[ip as usize] {
                Inst::BinaryExpr(expr) => {
                    let rhs = self.pop()?;
                    let lhs = self.pop()?;

                    self.stack.push(
                        Value::Int(
                            match expr {
                                ExprKind::Add => {
                                    lhs.as_int() + rhs.as_int()
                                },
                                ExprKind::Sub => {
                                    lhs.as_int() - rhs.as_int()
                                },
                                ExprKind::Mul => {
                                    lhs.as_int() * rhs.as_int()
                                },
                                ExprKind::Div => {
                                    lhs.as_int() / rhs.as_int()
                                },
                            }
                        )
                    );
                },
                Inst::Call(addr) => {
                    self.ret_stack.push(ip + 1);
                    self.jump(&labels, &mut ip, *addr)?;
                    continue;
                },
                Inst::Jump(jump, addr) => {
                    let result = if *jump != Jump::Unconditional {
                        self.pop()?.as_int()
                    } else {
                        0
                    };

                    if match jump {
                        Jump::Unconditional => true,
                        Jump::Equal => result == 0,
                        Jump::NotEqual => result != 0,
                        Jump::Greater => result == 1,
                        Jump::Lesser => result == 2,
                    } {
                        self.jump(&labels, &mut ip, *addr)?;
                        continue;
                    }
                },
                Inst::StackOp(stackop) => {
                    match stackop {
                        StackOp::Push(integer) => {
                            self.stack.push(Value::Int(*integer));
                        },
                        StackOp::Pop => {
                            self.pop()?;
                        },
                        StackOp::Dup => {
                            if !self.stack.is_empty() {
                                let value = self.pop()?;
                                for _ in 0..2 {
                                    self.stack.push(value);
                                }
                            }
                        },
                        StackOp::Rot | StackOp::Swap => {
                            let values = if *stackop == StackOp::Rot {
                                [self.pop()?, self.pop()?, self.pop()?].to_vec()
                            } else {
                                [self.pop()?, self.pop()?].to_vec()
                            };

                            for value in values {
                                self.stack.push(value);
                            }
                        },
                        StackOp::Dump => {
                            println!("{}", self.pop()?);
                        },
                        StackOp::Cmp => {
                            let rhs = self.pop()?.as_int();
                            let lhs = self.pop()?.as_int();

                            if lhs == rhs {
                                self.stack.push(Value::Int(0));
                            } else if lhs > rhs {
                                self.stack.push(Value::Int(1));
                            } else {
                                self.stack.push(Value::Int(2));
                            }
                        },
                    }
                },
                Inst::MemOp(op) => {
                    let addr = self.pop()?.as_int();

                    self.bound_check(addr)?;

                    match op {
                        MemOp::InsertStr(string) => {
                            for (offset, character) in string.chars().enumerate() {
                                self.memory[addr as usize + offset] = Value::Int(character as u8 as u32);
                            }
                        },
                        MemOp::Load => {
                            self.stack.push(self.memory[addr as usize]);
                        },
                        MemOp::Store => {
                            self.memory[addr as usize] = self.pop()?;
                        },
                    }
                },
                Inst::Syscall => {
                    let syscall = Syscall::from(self.pop()?.as_int());

                    match syscall {
                        Syscall::Read | Syscall::Write => {
                            let fd = self.pop()?.as_int();
                            let buf = self.pop()?.as_int();
                            let count = self.pop()?.as_int();

                            if let Err(err) = match syscall {
                                Syscall::Read => syscall::read(fd as usize, &mut self.memory[buf as usize..buf as usize + count as usize]),
                                Syscall::Write => syscall::write(fd as usize, &mut self.memory[buf as usize..buf as usize + count as usize]),
                                _ => unreachable!(),
                            } {
                                return Err(ErrorKind::Syscall(err.to_string()));
                            }
                        },
                        Syscall::Open => {
                            let ptr = self.pop()?.as_int() as usize;
                            let flags = self.pop()?.as_int() as usize;
                            let filename = self.to_string(&self.memory[ptr..ptr + self.strlen(ptr)]);

                            if let Err(status) = syscall::open(&filename, flags) {
                                return Err(ErrorKind::Syscall(format!("open failed with status {}", status)));
                            }
                        },
                        Syscall::Close => {
                            let fd = self.pop()?.as_int();

                            if let Err(status) = syscall::close(fd as usize) {
                                return Err(ErrorKind::Syscall(format!("close failed with status {}", status)));
                            }
                        },
                        Syscall::Unknown => {
                            return Err(ErrorKind::UnknownSyscall);
                        },
                    }
                },
                Inst::Halt => {
                    return Ok(());
                },
                Inst::Return => {
                    let addr = self.ret_stack.pop().ok_or(ErrorKind::StackUnderflow)?;

                    if addr >= instructions.len() as u32 {
                        return Err(ErrorKind::OutOfBounds);
                    } else {
                        ip = addr;
                        continue;
                    }
                },
                Inst::Label(_) => (),
            }

            if self.debug {
                log::info(&format!("======"));
                log::info(&format!("Stack: {:?}", self.stack));
                log::info(&format!("Return: {:?}", self.ret_stack));
                log::info(&format!("======"));

                loop {
                    let mut buf = String::new();

                    io::stdin().read_line(&mut buf).expect("failed to read stdin");

                    if &buf == "mem\n" {
                        println!("MEM: {:?}", &self.memory[0..30]);
                    } else {
                        break;
                    }
                }
            }

            ip += 1;
        }

        Ok(())
    }
}


