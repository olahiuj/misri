use crate::{
    instr::{ArithOp, Func, Instr, Operand, Program, RelOp},
    lexer::{Lexer, Token},
};

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    body: Vec<Instr>,
}

impl Parser {
    pub fn from(input: &str) -> Parser {
        Parser {
            lexer: Lexer::from(String::from(input)),
            body: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Program {
        let (token, lineno) = self.lexer.peek();
        match token {
            Token::TokFunc => {
                let fun = self.parse_func();
                let mut program = self.parse();
                program.push_front(fun);
                program
            }
            Token::TokEOF => Program::new(),
            Token::TokNl => {
                self.lexer.consume();
                self.parse()
            }
            token => panic!("parse error: {:?} at line: {lineno}", token),
        }
    }

    fn parse_func(&mut self) -> Func {
        self.lexer.consume();
        let name = self.parse_name();
        self.lexer.consume();
        self.lexer.consume();
        self.body = Vec::new();
        self.parse_body();
        Func {
            name,
            body: self.body.clone(),
            nreg: 0,
            id: 0,
        }
    }

    fn parse_instr(&mut self) -> Instr {
        let (token, lineno) = self.lexer.peek();
        let instr = match token {
            Token::TokLabel => {
                self.lexer.consume();
                let name = self.parse_name();
                self.lexer.consume();
                Instr::Label(name)
            }
            Token::TokIden(_) => {
                let x = self.parse_operand();
                self.lexer.consume();
                let (token, lineno) = self.lexer.peek();
                match token {
                    Token::TokAmp => {
                        self.lexer.consume();
                        let y = self.parse_operand();
                        Instr::Deref(x, y)
                    }
                    Token::TokStar => {
                        self.lexer.consume();
                        let y = self.parse_operand();
                        Instr::Load(x, y)
                    }
                    Token::TokCall => {
                        self.lexer.consume();
                        let name = self.parse_name();
                        Instr::Call {
                            x,
                            name,
                            id: Default::default(),
                        }
                    }
                    Token::TokIden(_) | Token::TokSharp => {
                        let y = self.parse_operand();
                        let (token, _) = self.lexer.peek();
                        match token {
                            Token::TokAdd | Token::TokSub | Token::TokStar | Token::TokDiv => {
                                let op = self.parse_arith_op();
                                let z = self.parse_operand();
                                Instr::Arith(x, y, op, z)
                            }
                            Token::TokNl => Instr::Assign(x, y),
                            _ => Instr::Assign(x, y),
                        }
                    }
                    token => panic!("parse error: {:?} at line: {lineno}", token),
                }
            }
            Token::TokStar => {
                self.lexer.consume();
                let lhs = self.parse_operand();
                self.lexer.consume();
                let rhs = self.parse_operand();
                Instr::Store(lhs, rhs)
            }
            Token::TokGoto => {
                self.lexer.consume();
                Instr::new_goto(&self.parse_name())
            }
            Token::TokIf => {
                self.lexer.consume();
                let x = self.parse_operand();
                let op = self.parse_rel_op();
                let y = self.parse_operand();
                self.lexer.consume();
                let name = self.parse_name();
                Instr::Cond {
                    x,
                    op,
                    y,
                    name,
                    id: Default::default(),
                }
            }
            Token::TokReturn => {
                self.lexer.consume();
                Instr::Return(self.parse_operand())
            }
            Token::TokDec => {
                self.lexer.consume();
                let tar = self.parse_operand();
                let size = self.parse_int();
                Instr::Dec(tar, size)
            }
            Token::TokArg => {
                self.lexer.consume();
                Instr::Arg(self.parse_operand())
            }
            Token::TokParam => {
                self.lexer.consume();
                Instr::Param(self.parse_operand())
            }
            Token::TokRead => {
                self.lexer.consume();
                Instr::Read(self.parse_operand())
            }
            Token::TokWrite => {
                self.lexer.consume();
                Instr::Write(self.parse_operand())
            }
            token => panic!("parse error: {:?} at line: {lineno}", token),
        };
        match self.lexer.consume().0 {
            Token::TokNl | Token::TokEOF => (),
            _ => panic!("parse error: {:?} at line: {lineno}", token),
        }
        instr
    }

    fn parse_operand(&mut self) -> Operand {
        let (token, lineno) = self.lexer.consume();
        match token {
            Token::TokSharp => Operand::Imm(self.parse_int()),
            Token::TokIden(name) => Operand::from(name),
            token => panic!("parse error: {:?} at line: {lineno}", token),
        }
    }

    fn parse_int(&mut self) -> i64 {
        let mut sign: i64 = 1;
        if self.lexer.peek().0 == Token::TokSub {
            self.lexer.consume();
            sign = -1
        }
        let (token, lineno) = self.lexer.consume();
        match token {
            Token::TokInt(int) => int * sign,
            token => panic!("parse error: {:?} at line: {lineno}", token),
        }
    }

    fn parse_rel_op(&mut self) -> RelOp {
        let (token, lineno) = self.lexer.consume();
        match token {
            Token::TokLT => RelOp::LT,
            Token::TokLE => RelOp::LE,
            Token::TokGT => RelOp::GT,
            Token::TokGE => RelOp::GE,
            Token::TokEQ => RelOp::EQ,
            Token::TokNE => RelOp::NE,
            token => panic!("parse error: {:?} at line: {lineno}", token),
        }
    }

    fn parse_arith_op(&mut self) -> ArithOp {
        let (token, lineno) = self.lexer.consume();
        match token {
            Token::TokAdd => ArithOp::Add,
            Token::TokSub => ArithOp::Sub,
            Token::TokStar => ArithOp::Mul,
            Token::TokDiv => ArithOp::Div,
            token => panic!("parse error: {:?} at line: {lineno}", token),
        }
    }

    fn parse_name(&mut self) -> String {
        let (token, lineno) = self.lexer.consume();
        match token {
            Token::TokIden(name) => name,
            token => panic!("parse error: {:?} at line: {lineno}", token),
        }
    }

    fn parse_body(&mut self) {
        let (token, lineno) = self.lexer.peek();
        match token {
            Token::TokFunc | Token::TokEOF | Token::TokNl => (),
            Token::TokIf
            | Token::TokLabel
            | Token::TokIden(_)
            | Token::TokStar
            | Token::TokGoto
            | Token::TokReturn
            | Token::TokWrite
            | Token::TokRead
            | Token::TokParam
            | Token::TokDec
            | Token::TokArg => {
                let instr = self.parse_instr();
                self.body.push(instr);
                self.parse_body()
            }
            token => panic!("parse error: {:?} at line: {lineno}", token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instr() {
        let mut parser = Parser::from(
            "x := y
             x := y + z
             x := y - z
             x := y * z
             x := y / z
             x := &y
             x := *y
             *x := y
             GOTO wjp
             LABEL wjp :
             IF x < y GOTO wjp
             RETURN x
             DEC arr 24
             ARG x
             y := CALL foo
             PARAM x
             READ x
             WRITE x",
        );
        assert_eq!(
            parser.parse_instr(),
            Instr::Assign(Operand::from("x"), Operand::from("y"))
        );
        assert_eq!(
            parser.parse_instr(),
            Instr::Arith(
                Operand::from("x"),
                Operand::from("y"),
                ArithOp::Add,
                Operand::from("z")
            )
        );
        assert_eq!(
            parser.parse_instr(),
            Instr::Arith(
                Operand::from("x"),
                Operand::from("y"),
                ArithOp::Sub,
                Operand::from("z")
            )
        );
        assert_eq!(
            parser.parse_instr(),
            Instr::Arith(
                Operand::from("x"),
                Operand::from("y"),
                ArithOp::Mul,
                Operand::from("z")
            )
        );
        assert_eq!(
            parser.parse_instr(),
            Instr::Arith(
                Operand::from("x"),
                Operand::from("y"),
                ArithOp::Div,
                Operand::from("z")
            )
        );
        assert_eq!(
            parser.parse_instr(),
            Instr::Deref(Operand::from("x"), Operand::from("y"))
        );
        assert_eq!(
            parser.parse_instr(),
            Instr::Load(Operand::from("x"), Operand::from("y"))
        );
        assert_eq!(
            parser.parse_instr(),
            Instr::Store(Operand::from("x"), Operand::from("y"))
        );
        assert_eq!(parser.parse_instr(), Instr::new_goto("wjp"));
        assert_eq!(parser.parse_instr(), Instr::Label(String::from("wjp")));
        assert_eq!(
            parser.parse_instr(),
            Instr::Cond {
                x: Operand::from("x"),
                op: RelOp::LT,
                y: Operand::from("y"),
                name: String::from("wjp"),
                id: Default::default()
            }
        );
        assert_eq!(parser.parse_instr(), Instr::Return(Operand::from("x")));
        assert_eq!(parser.parse_instr(), Instr::Dec(Operand::from("arr"), 24));
        assert_eq!(parser.parse_instr(), Instr::Arg(Operand::from("x")));
        assert_eq!(
            parser.parse_instr(),
            Instr::Call {
                x: Operand::from("y"),
                name: String::from("foo"),
                id: Default::default()
            }
        );
        assert_eq!(parser.parse_instr(), Instr::Param(Operand::from("x")));
        assert_eq!(parser.parse_instr(), Instr::Read(Operand::from("x")));
        assert_eq!(parser.parse_instr(), Instr::Write(Operand::from("x")));
    }

    #[test]
    fn test_func() {
        let mut parser = Parser::from(
            "FUNCTION fact :
             PARAM v1
             IF v1 == #1 GOTO label1
             GOTO label2
             LABEL label1 :
             RETURN v1
             LABEL label2 :
             t1 := v1 - #1
             ARG t1
             t2 := CALL fact
             t3 := v1 * t2
             RETURN t3",
        );
        let func = parser.parse_func();
        assert_eq!(func.name, String::from("fact"));
        assert_eq!(func.body.len(), 11);
        assert_eq!(
            func.body,
            Vec::from([
                Instr::Param(Operand::from("v1")),
                Instr::Cond {
                    x: Operand::from("v1"),
                    op: RelOp::EQ,
                    y: Operand::from(1),
                    name: String::from("label1"),
                    id: Default::default()
                },
                Instr::new_goto("label2"),
                Instr::Label(String::from("label1")),
                Instr::Return(Operand::from("v1")),
                Instr::Label(String::from("label2")),
                Instr::Arith(
                    Operand::from("t1"),
                    Operand::from("v1"),
                    ArithOp::Sub,
                    Operand::from(1)
                ),
                Instr::Arg(Operand::from("t1")),
                Instr::Call {
                    x: Operand::from("t2"),
                    name: String::from("fact"),
                    id: Default::default()
                },
                Instr::Arith(
                    Operand::from("t3"),
                    Operand::from("v1"),
                    ArithOp::Mul,
                    Operand::from("t2")
                ),
                Instr::Return(Operand::from("t3")),
            ])
        )
    }

    #[test]
    fn test_program() {
        let mut parser = Parser::from(
            "FUNCTION add :
             PARAM v1
             t2 := *v1
             t7 := v1 + #4
             t3 := *t7
             t1 := t2 + t3
             RETURN t1

             FUNCTION main :
             DEC v3 8
             t9 := &v3
             *t9 := #1
             t12 := t10 + #4
             *t12 := #2
             ARG t10
             t14 := CALL add
             v2 := t14
             WRITE v2
             RETURN #0",
        );
        let program = parser.parse();
        assert_eq!(program.funcs.len(), 2);
    }

    #[test]
    fn test_nl() {
        let mut parser = Parser::from(
            "FUNCTION main :
             n_i_func_97_i_113 := #0
             *t_280_at_42_ := n_i_func_97_i_113",
        );
        parser.parse();
    }
}
