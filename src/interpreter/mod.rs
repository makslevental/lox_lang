use crate::environment::{Environment, Object};
use crate::interpreter::callable::Clock;
use crate::lexer;
use crate::lexer::Operator;
use crate::parser::ast::{Expr, ExprData, ExprVisitor, Literal, Stmt, StmtData, StmtVisitor};
use std::cell::RefCell;
use std::option::Option::Some;
use std::rc::Rc;

pub mod callable;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    globals: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();
        globals.define("clock", Object::C(Rc::new(Clock {})));
        let globals = Rc::new(RefCell::new(globals));
        Self {
            environment: globals.clone(),
            globals: globals,
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Option<Object> {
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
        let previous_env =
            std::mem::replace(&mut self.environment, Rc::new(RefCell::new(environment)));
        for stmt in stmts {
            self.execute(stmt)
        }
        std::mem::replace(&mut self.environment, previous_env);
    }
}

impl ExprVisitor for Interpreter {
    type Result = Object;

    fn visit_literal(&mut self, expr: &Literal) -> Option<Self::Result> {
        Some(Object::L(expr.clone()))
    }

    fn visit_unary(&mut self, expr: &Expr) -> Option<Self::Result> {
        if let Expr::Unary { operator, right } = expr {
            let right = self.evaluate(right);
            if *operator == Operator::Minus {
                return right.map(|r|
                    match r {
                        Object::L(Literal::Float(l)) => Object::L(Literal::Float(-l)),
                        _ => panic!("{:?}", r)
                    }
                )
            } else if *operator == Operator::Not {
                return right.map(|r|
                    match r {
                        Object::L(Literal::Bool(b)) => Object::L(Literal::Bool(!b)),
                        _ => panic!("{:?}", r)
                    }
                )
            } else {
                panic!("{:?}", operator)
            };
        }
        panic!("{:?}", expr)
    }

    fn visit_binary(&mut self, expr: &Expr) -> Option<Self::Result> {
        if let Expr::Binary {
            left,
            operator,
            right,
        } = expr
        {
            if let Some(Object::L(Literal::String(left))) = self.evaluate(left) {
                if let Some(Object::L(Literal::String(right))) = self.evaluate(right) {
                    return Some(Object::L(match operator {
                        Operator::Plus => Literal::String(left + right.as_str()),
                        _ => panic!("{:?}", operator),
                    }));
                }
            }
            if let Some(Object::L(Literal::Float(left))) = self.evaluate(left) {
                if let Some(Object::L(Literal::Float(right))) = self.evaluate(right) {
                    return Some(Object::L(match operator {
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
                        _ => panic!("{:?}", operator),
                    }));
                }
            }
        }
        panic!("{:?}", expr)
    }

    fn visit_logical(&mut self, expr: &Expr) -> Option<Self::Result> {
        if let Expr::Logical {
            left,
            operator,
            right,
        } = expr
        {
            if let Some(Object::L(Literal::Bool(left))) = self.evaluate(left) {
                if let Some(Object::L(Literal::Bool(right))) = self.evaluate(right) {
                    return Some(Object::L(match operator {
                        Operator::And => Literal::Bool(left && right),
                        Operator::Or => Literal::Bool(left || right),
                        _ => panic!("{:?}", operator),
                    }));
                }
            }
        }
        panic!("{:?}", expr)
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Option<Self::Result> {
        unimplemented!()
    }

    fn visit_assign(&mut self, expr: &Expr) -> Option<Self::Result> {
        if let Expr::Assign { name, value } = expr {
            if let lexer::Token::Identifier(name) = name {
                let value = self.evaluate(value);
                self.environment
                    .borrow_mut()
                    .assign(name, value.clone().unwrap());
                return value;
            }
        }
        panic!("{:?}", expr)
    }

    fn visit_variable(&mut self, expr: &Expr) -> Option<Self::Result> {
        if let Expr::Variable { name } = expr {
            return Some(self.environment.borrow_mut().get(name))
        }
        panic!("{:?}", expr)
    }

    fn visit_call(&mut self, expr: &Expr) -> Option<Self::Result> {
        if let Expr::Call { callee, arguments } = expr {
            if let Some(Object::C(callee)) = self.evaluate(callee) {
                let mut args = Vec::new();
                for argument in arguments {
                    args.push(self.evaluate(argument))
                }
                if args.len() != callee.arity() {
                    panic!("wrong number of args {:?} {:?}", arguments.len(), callee.arity())
                }
                return callee.call(self, args.into_iter().map(|o| o.unwrap()).collect());
            }
        }
        panic!("{:?}", expr)
    }
}

impl StmtVisitor for Interpreter {
    fn visit_expr_stmt(&mut self, stmt: &Stmt) {
        if let Stmt::Expr(expr) = stmt {
            self.evaluate(expr);
        } else {
            panic!("{:?}", stmt)
        }
    }

    fn visit_print_stmt(&mut self, stmt: &Stmt) {
        if let Stmt::Print(expr) = stmt {
            match self.evaluate(expr).unwrap() {
                Object::L(Literal::Float(l)) => println!("{:?}", l),
                Object::L(Literal::Bool(l)) => println!("{:?}", l),
                Object::L(Literal::String(l)) => println!("{:?}", l),
                Object::L(Literal::Nil(l)) => println!("{:?}", l),
                Object::C(c) => println!("{:?}", c)
            }
        } else {
            panic!("{:?}", stmt)
        }
    }

    fn visit_var_stmt(&mut self, stmt: &Stmt) {
        if let Stmt::Variable { name, initializer } = stmt {
            if let lexer::Token::Identifier(name) = name {
                let initializer = initializer.as_ref();
                let mut value = Object::L(Literal::Nil(()));
                if *initializer != Expr::L(Literal::Nil(())) {
                    value = self.evaluate(initializer).unwrap();
                }

                self.environment.borrow_mut().define(name, value);
                return;
            }
        }
        panic!("{:?}", stmt)
    }

    fn visit_block(&mut self, stmt: &Stmt) {
        if let Stmt::Block(stmts) = stmt {
            self.execute_block(
                stmts,
                Environment {
                    values: Default::default(),
                    enclosing: Some(self.environment.clone()),
                },
            )
        }
    }

    fn visit_if(&mut self, stmt: &Stmt) {
        if let Stmt::If {
            condition,
            then_branch,
            else_branch,
        } = stmt
        {
            if let Some(Object::L(Literal::Bool(true))) = self.evaluate(condition) {
                self.execute(then_branch)
            } else if let Some(else_branch) = else_branch {
                self.execute(else_branch)
            }
        } else {
            panic!("{:?}", stmt);
        }
    }

    fn visit_while(&mut self, stmt: &Stmt) {
        if let Stmt::While { condition, body } = stmt {
            while let Some(Object::L(Literal::Bool(true))) = self.evaluate(condition) {
                self.execute(body);
            }
        } else {
            panic!("{:?}", stmt);
        }
    }

    fn visit_function(&mut self, stmt: &Stmt) {
        if let Stmt::Function { name, parameters, ..} = stmt {
            if let lexer::Token::Identifier(name_str) = name {
                let s = stmt.clone();
                let f = callable::Function {
                    declaration: s
                };
                self.environment.borrow_mut().define(name_str, Object::C(Rc::new(f)))
            }
        } else {
            panic!("{:?}", stmt);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::lexer::{lexer, Operator, Token};
    use crate::parser::ast::{Expr, Literal, Stmt};
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

        Interpreter::new().interpret(&[Stmt::Expr(expr)]);

        let x = 1.0;
        let y = 2.0;
        let expr = Box::new(Expr::Binary {
            left: Box::new(Expr::L(Literal::Float(x))),
            operator: Operator::And,
            right: Box::new(Expr::L(Literal::Float(y))),
        });

        Interpreter::new().interpret(&[Stmt::Expr(expr)]);
    }

    #[test]
    fn interpret_var_stmt() {
        let mut i = Interpreter::new();

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

        Interpreter::new().interpret(&[print]);
    }

    #[test]
    fn interpret_var() {
        let input: Vec<char> = "
            var a = \"global a\";
            print a;
        "
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        Interpreter::new().interpret(e.as_ref());
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
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        Interpreter::new().interpret(e.as_ref());
    }

    #[test]
    fn interpret_if() {
        let input: Vec<char> = r#"
            var a = 5;
            var b = 6;
            if (a < b) {
                print "hello";
            } else {
                print "world";
            }

            if (a <= b) {
                print "second";
            }

            print a;
            if (a < 10) {
                print a;
                a = 11;
            }
            print a;
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        Interpreter::new().interpret(e.as_ref());
    }

    #[test]
    fn interpret_comparison() {
        let input: Vec<char> = r#"
            var a = 5;
            var b = 6;
            print a;
            print b;
            print a < b;
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        println!("{:#?}", e);
        Interpreter::new().interpret(e.as_ref());
    }

    #[test]
    fn interpret_logical() {
        let input: Vec<char> = r#"
            var a = 5;
            var b = 6;
            print a;
            print b;
            print a < b or false;
            print a < b and false;
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        println!("{:?}", e);
        Interpreter::new().interpret(e.as_ref());
    }

    #[test]
    fn interpret_assign() {
        let input: Vec<char> = r#"
            var a = 5;
            print a;
            a = a + 1;
            print a;
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        Interpreter::new().interpret(e.as_ref());
    }

    #[test]
    fn interpret_while() {
        let input: Vec<char> = r#"
            var a = 5;
            print a;
            while (a < 10) {
                print a;
                a = a + 1;
            }
            print a;
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        Interpreter::new().interpret(e.as_ref());
    }

    #[test]
    fn intepret_for() {
        let input: Vec<char> = r#"
            for (var i = 0; i < 10; i = i + 1) {
                var j = 2;
                print j*i;
            }
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        Interpreter::new().interpret(e.as_ref());
    }

    #[test]
    fn function() {
        let input: Vec<char> = r#"
            fun bob(a, b, c) {
                var d = 1;
                print a + b + c + d;
            }
        "#
            .chars()
            .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        Interpreter::new().interpret(e.as_ref());
    }
}
