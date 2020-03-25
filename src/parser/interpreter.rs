use crate::environment::Environment;
use crate::lexer;
use crate::lexer::Operator;
use crate::parser::ast::Literal::Nil;
use crate::parser::ast::{Expr, ExprData, ExprVisitor, Literal, Stmt, StmtData, StmtVisitor};
use pom::range::Bound::Excluded;


pub struct Interpreter {
    pub environment: Environment,
}

impl Interpreter {
    pub fn evaluate(&mut self, expr: &Expr) -> Literal {
        expr.accept(self)
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.execute(stmt)
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) {
        stmt.accept(self)
    }

    pub fn execute_block(&mut self, stmts: &[Stmt], environment: Environment) {
        let previous_env = std::mem::replace(&mut self.environment, environment);
        for stmt in stmts {
            self.execute(stmt)
        }
        std::mem::replace(&mut self.environment, previous_env);
    }
}

impl ExprVisitor for Interpreter {
    type Result = Literal;

    fn visit_literal(&mut self, expr: &Literal) -> Self::Result {
        (*expr).clone()
    }

    fn visit_unary(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Unary { operator, right } = expr {
            let right = self.evaluate(right);
            return if *operator == Operator::Minus {
                match right {
                    Literal::Float(l) => Literal::Float(-l),
                    _ => panic!(),
                }
            } else if *operator == Operator::Not {
                match right {
                    Literal::Bool(b) => Literal::Bool(!b),
                    _ => panic!(),
                }
            } else {
                panic!()
            };
        }
        panic!()
    }

    fn visit_binary(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Binary {
            left,
            operator,
            right,
        } = expr
        {
            if let Literal::String(left) = self.evaluate(left) {
                if let Literal::String(right) = self.evaluate(right) {
                    return match operator {
                        Operator::Plus => Literal::String(left + right.as_str()),
                        _ => panic!(),
                    };
                }
            }

            if let Literal::Float(left) = self.evaluate(left) {
                if let Literal::Float(right) = self.evaluate(right) {
                    return match operator {
                        Operator::Minus => Literal::Float(left - right),
                        Operator::Plus => Literal::Float(left + right),
                        Operator::Slash => Literal::Float(left / right),
                        Operator::Star => Literal::Float(left * right),
                        Operator::NotEqual => Literal::Bool(left != right),
                        Operator::Equal => Literal::Bool(left == right),
                        Operator::GreaterThanOrEqual => Literal::Bool(left >= right),
                        Operator::LessThanOrEqual => Literal::Bool(left <= right),
                        Operator::GreaterThan => Literal::Bool(left > right),
                        Operator::LessThan => Literal::Bool(left < right),
                        Operator::And => Literal::Bool(left > 0.0 && right > 0.0),
                        Operator::Or => Literal::Bool(left > 0.0 || right > 0.0),
                        _ => panic!(),
                    };
                }
            }
        }
        panic!()
    }

    fn visit_logical(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Binary {
            left,
            operator,
            right,
        } = expr
        {
            if let Literal::Bool(left) = self.evaluate(left) {
                if let Literal::Bool(right) = self.evaluate(right) {
                    return match operator {
                        Operator::And => Literal::Bool(left && right),
                        Operator::Or => Literal::Bool(left || right),
                        _ => panic!(),
                    };
                }
            }
        }
        panic!()
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Self::Result {
        unimplemented!()
    }

    fn visit_assign(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Assign { name, value } = expr {
            if let lexer::Token::Identifier(name) = name {
                let value = self.evaluate(expr);
                self.environment.assign(name, value.clone());
                return value;
            }
        }
        panic!()
    }

    fn visit_variable(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Variable { name } = expr {
            return self.environment.get(name);
        }
        panic!()
    }
}

impl StmtVisitor for Interpreter {
    fn visit_expr_stmt(&mut self, stmt: &Stmt) {
        if let Stmt::Expr(expr) = stmt {
            self.evaluate(expr);
        } else {
            panic!()
        }
    }

    fn visit_print_stmt(&mut self, stmt: &Stmt) {
        if let Stmt::Print(expr) = stmt {
            match self.evaluate(expr) {
                Literal::Float(l) => println!("{:?}", l),
                Literal::Bool(l) => println!("{:?}", l),
                Literal::String(l) => println!("{:?}", l),
                Literal::Nil(l) => println!("{:?}", l),
            }
        } else {
            panic!()
        }
    }

    fn visit_var_stmt(&mut self, stmt: &Stmt) {
        if let Stmt::Variable { name, initializer } = stmt {
            if let lexer::Token::Identifier(name) = name {
                let initializer = initializer.as_ref();
                let mut value = Literal::Nil(());
                if *initializer != Expr::L(Literal::Nil(())) {
                    value = self.evaluate(initializer);
                }
                self.environment.define(name, value);
                return;
            }
        }
        panic!()
    }

    fn visit_block(&mut self, stmt: &Stmt) {
        if let Stmt::Block(stmts) = stmt {
            self.execute_block(stmts, Environment { values: Default::default(), enclosing: Some(Box::new(self.environment.clone())) })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::environment::Environment;
    use crate::lexer::{Operator, Token, lexer};
    use crate::parser::ast::{Expr, Literal, Stmt};
    use crate::parser::interpreter::Interpreter;
    use crate::parser::parser::Parser;


    #[test]
    fn string() {
        let x = String::from("x");
        let y = String::from("y");
        let expr = Box::new(Expr::Binary {
            left: Box::new(Expr::L(Literal::String(x))),
            operator: Operator::Plus,
            right: Box::new(Expr::L(Literal::String(y))),
        });

        (Interpreter { environment: Default::default() }).interpret(&[Stmt::Expr(expr)]);

        let x = 1.0;
        let y = 2.0;
        let expr = Box::new(Expr::Binary {
            left: Box::new(Expr::L(Literal::Float(x))),
            operator: Operator::And,
            right: Box::new(Expr::L(Literal::Float(y))),
        });

        (Interpreter { environment: Default::default() }).interpret(&[Stmt::Expr(expr)]);
    }

    #[test]
    fn interpret_var_stmt() {
        let mut i = Interpreter {
            environment: Default::default(),
        };

        let name = Token::Identifier("z".to_string());
        let st = Stmt::Variable {
            name: name.clone(),
            initializer: Box::new(Expr::L(Literal::String("this is z".to_string()))),
        };
        i.interpret(&[st]);

        let print = Stmt::Print(Box::from(Expr::Variable { name: name.clone() }));
        i.interpret(&[print]);

        let st = Stmt::Variable {
            name: name.clone(),
            initializer: Box::new(Expr::L(Literal::Float(1.0))),
        };
        i.interpret(&[st]);
        let print = Stmt::Print(Box::from(Expr::Variable { name: name.clone() }));
        i.interpret(&[print]);
    }

    #[test]
    fn interpret_print_stmt() {
        let x = String::from("x");
        let y = String::from("y");
        let print = Stmt::Print(Box::new(Expr::Binary {
            left: Box::new(Expr::L(Literal::String(x))),
            operator: Operator::Plus,
            right: Box::new(Expr::L(Literal::String(y))),
        }));

        (Interpreter {
            environment: Default::default(),
        })
            .interpret(&[print]);
    }

    #[test]
    fn interpret_var() {
        let input: Vec<char> = "
            var a = \"global a\";
            print a;
        ".chars().collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse_stmts();
        (Interpreter {
            environment: Default::default(),
        })
            .interpret(e.as_ref());
    }

    #[test]
    fn interpret_blocks() {
        let input: Vec<char> = r#"
            var a = "global a";
            var b = "global b";
            var c = "global c";
            {
              var a = "outer a";
              var b = "outer b";
              {
                var a = "inner a";
                print a;
                print b;
                print c;
              }
              print a;
              print b;
              print c;
            }
            print a;
            print b;
            print c;
        "#.chars().collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse_stmts();
        println!("{:#?}", e);
        (Interpreter {
            environment: Default::default(),
        })
            .interpret(e.as_ref());
    }
}
