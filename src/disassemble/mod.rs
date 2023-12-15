use lib_stacked::*;

pub fn disassemble(instructions: Vec<Inst>) {
    for inst in instructions {
        match inst {
            Inst::Label(_) => println!("0x4C {}", inst),

            Inst::Call(_) => println!("0x2F {}", inst),

            Inst::Jump(Jump::Unconditional, _) => println!("0x6A {}", inst),
            Inst::Jump(Jump::Equal, _) => println!("0x6B {}", inst),
            Inst::Jump(Jump::NotEqual, _) => println!("0x6E {}", inst),
            Inst::Jump(Jump::Greater, _) => println!("0x6C {}", inst),
            Inst::Jump(Jump::Lesser, _) => println!("0x6D {}", inst),

            Inst::MemOp(MemOp::Load) => println!("0x8A {}", inst),
            Inst::MemOp(MemOp::Store) => println!("0x8B {}", inst),
            Inst::MemOp(MemOp::InsertStr(_)) => println!("0x8C {}", inst),

            Inst::StackOp(StackOp::Push(_)) => println!("0x01 {}", inst),
            Inst::StackOp(StackOp::Pop) => println!("0x02 {}", inst),
            Inst::StackOp(StackOp::Dup) => println!("0x05 {}", inst),
            Inst::StackOp(StackOp::Swap) => println!("0x06 {}", inst),
            Inst::StackOp(StackOp::Rot) => println!("0x07 {}", inst),
            Inst::StackOp(StackOp::Dump) => println!("0x03 {}", inst),
            Inst::StackOp(StackOp::Cmp) => println!("0x43 {}", inst),

            Inst::BinaryExpr(ExprKind::Add) => println!("0x28 {}", inst),
            Inst::BinaryExpr(ExprKind::Sub) => println!("0x29 {}", inst),
            Inst::BinaryExpr(ExprKind::Mul) => println!("0x2A {}", inst),
            Inst::BinaryExpr(ExprKind::Div) => println!("0x2B {}", inst),

            Inst::Syscall => println!("0x53 {}", inst),
            Inst::Return => println!("0x0D {}", inst),
            Inst::Halt => println!("0x04 {}", inst),
        }
    }
}

