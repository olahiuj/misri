use std::io;

use crate::{
    env::Env,
    instr::{ArithOp, Instr::*, Program, RelOp},
    value::Value,
};

pub fn exec(program: &Program) {
    let mut env = Env::new(&program);
    loop {
        let instr = program.fetch(env.top_frame());
        match instr {
            IrArith(x, y, op, z) => {
                env.pc_advance();
                let vy = env.get(y);
                let vz = env.get(z);
                let value = match op {
                    ArithOp::OpAdd => vy + vz,
                    ArithOp::OpSub => vy - vz,
                    ArithOp::OpMul => vy * vz,
                    ArithOp::OpDiv => vy / vz,
                };
                env.set(x, value)
            }
            IrAssign(x, y) => {
                env.pc_advance();
                env.set(x, env.get(y))
            }
            IrDeref(x, y) => {
                env.pc_advance();
                env.set(x, env.get(y))
            }
            IrStore(x, y) => {
                env.pc_advance();
                env.get(x).store(env.get(y))
            }
            IrLoad(x, y) => {
                env.pc_advance();
                env.set(x, env.get(y).load())
            }
            IrArg(x) => {
                env.pc_advance();
                env.push_arg(env.get(x))
            }
            IrParam(x) => {
                env.pc_advance();
                let value = env.pop_arg();
                env.set(x, value)
            }
            IrLabel(_) => env.pc_advance(),
            IrRead(x) => {
                env.pc_advance();
                let buf = &mut String::new();
                io::stdin().read_line(buf).expect("input error");
                let int: i32 = buf.trim().parse().expect("input error");
                env.set(x, Value::new_int(int))
            }
            IrWrite(x) => {
                env.pc_advance();
                let value = env.get(x);
                println!("{value}")
            }
            IrDec(x, size) => {
                env.pc_advance();
                env.set(x, Value::new_ptr(size as usize))
            }
            IrCall { id, .. } => {
                env.push_frame(id);
            }
            IrReturn(x) => {
                if env.top_frame().func == program.entry {
                    return;
                }
                let value = env.get(x);
                env.pop_frame();
                let func = &program.funcs[env.top_frame().func];
                match &func.body[env.pc()] {
                    IrCall { x, .. } => env.set(x.clone(), value),
                    _ => panic!("return error"),
                }
                env.pc_advance()
            }
            IrGoto { id, .. } => env.pc_set(id),
            IrCond { x, op, y, id, .. } => {
                let vx = env.get(x);
                let vy = env.get(y);
                let jmp = match op {
                    RelOp::OpLT => vx < vy,
                    RelOp::OpLE => vx <= vy,
                    RelOp::OpGT => vx > vy,
                    RelOp::OpGE => vx >= vy,
                    RelOp::OpEQ => vx == vy,
                    RelOp::OpNE => vx != vy,
                };
                if jmp {
                    env.pc_set(id);
                } else {
                    env.pc_advance();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    use super::*;

    #[test]
    fn test_exec() {
        let mut parser = Parser::from(
            "FUNCTION main :
             x := #114 * #514
             y := #0 - x
             WRITE y
             RETURN #0
             ",
        );
        let mut program = parser.parse();
        program.init();
        exec(&program);
    }

    #[test]
    fn test_arg() {
        let mut parser = Parser::from(
            "FUNCTION id :
             PARAM n
             RETURN n

             FUNCTION main :
             ARG #114
             x := CALL id
             ARG #514
             y := CALL id
             WRITE x
             WRITE y
             RETURN #0
             ",
        );
        let mut program = parser.parse();
        program.init();
        exec(&program);
    }

    #[test]
    fn test_fib() {
        let mut parser = Parser::from(
            " FUNCTION fib :
             PARAM n
             IF n != #0 GOTO br1
             RETURN #0
             LABEL br1 :
             IF n != #1 GOTO br2
             RETURN #1
             LABEL br2 :
             t1 := n - #1
             ARG t1
             r1 := CALL fib
             t2 := n - #2
             ARG t2
             r2 := CALL fib
             u := r1 + r2
             RETURN u
 
             FUNCTION main :
             READ n
             ARG n
             s := CALL fib
             WRITE s
             RETURN #0
            ",
        );
        let mut program = parser.parse();
        program.init();
        exec(&program);
    }
}
