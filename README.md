
# Executable format layout

header | program


# Instruction Set Reference

### Push
Push a value onto the stack.
| Type       | OpCode | Args    |
| ---------- | ------ | ------- |
| push [u32] | 0x01   | [u8; 4] |

### Pop
Pop a value of the stack.
| Type      | OpCode | Args    |
| --------- | ------ | ------- |
| Pop       | 0x02   | None    |

### Dup
Duplicate the top of the stack.
| Type      | OpCode | Args    |
| --------- | ------ | ------- |
| Dup       | 0x05   | None    |

### Swap
Swap the top values of the stack.
| Type      | OpCode | Args    |
| --------- | ------ | ------- |
| Swap      | 0x06   | None    |

### Dump
Dumps the top of the stack to stdout.
| Type      | OpCode | Args    |
| --------- | ------ | ------- |
| Dump      | 0x03   | None    |

### Halt
Halts the execution of the program.
| Type      | OpCode | Args    |
| --------- | ------ | ------- |
| Halt      | 0x04   | None    |

### Binary Expr
Perform a binary expression on the stack.
| Type      | OpCode | Args    |
| --------- | ------ | ------- |
| Add       | 0x28   | None    |
| Sub       | 0x29   | None    |
| Mul       | 0x2A   | None    |
| Div       | 0x2B   | None    |

### Jump
Jump to a label.
| Type       | OpCode | Args    |
| ---------- | ------ | ------- |
| jump [u32] | 0x6A   | [u8; 4] |

### Label
Define a label with the specified u32 as identifier.
| Type       | OpCode | Args    |
| ---------- | ------ | ------- |
| labl [u32] | 0x4C   | [u8; 4] |

# Error Reference

### OutOfBounds
This error trigger when you try to access memory out of bounds
or you set the instruction pointer to an address out of bounds
via the return instruction.


