use crate::lexer;
use crate::lexer::Operator;
use crate::parser::ast;
use crate::parser::ast::Stmt;
use crate::parser::interpreter::Interpreter;
use crate::parser::ast::Stmt::Block;

pub struct Parser {
    tokens: Vec<lexer::Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<lexer::Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse_expr(&mut self) -> ast::Expr {
        self.expression()
    }

    pub fn parse_stmts(&mut self) -> Vec<ast::Stmt> {
        let mut statements: Vec<ast::Stmt> = Vec::new();
        while self.current < self.tokens.len() {
            statements.push(self.delaration());
        }
        statements
    }

    pub fn statement(&mut self) -> ast::Stmt {
        if self.tokens.get(self.current).unwrap().clone() == lexer::Token::Print {
            self.current += 1;
            return self.print();
        } else if self.tokens.get(self.current).unwrap().clone() == lexer::Token::LeftBrace {
            self.current += 1;
            return Block(self.block());
        }
        self.expr_stmt()
    }

    pub fn block(&mut self) -> Vec<ast::Stmt> {
        let mut statements = Vec::new();
        while self.current < self.tokens.len()
            && self.tokens.get(self.current).unwrap().clone() != lexer::Token::RightBrace
        {
            statements.push(self.delaration());
        }
        self.consume(lexer::Token::RightBrace);
        statements
    }

    pub fn delaration(&mut self) -> ast::Stmt {
        if self.tokens.get(self.current).unwrap().clone() == lexer::Token::Var {
            self.current += 1;
            return self.var_decl();
        }
        self.statement()
    }

    pub fn print(&mut self) -> ast::Stmt {
        let value = self.expression();
        self.consume(lexer::Token::Semicolon);
        ast::Stmt::Print(Box::new(value))
    }

    pub fn expr_stmt(&mut self) -> ast::Stmt {
        let expr = self.expression();
        self.consume(lexer::Token::Semicolon);
        ast::Stmt::Expr(Box::new(expr))
    }

    pub fn var_decl(&mut self) -> ast::Stmt {
        if let lexer::Token::Identifier(name) = self.tokens.get(self.current).unwrap().clone() {
            self.current += 1;
            if lexer::Token::Assign == self.tokens.get(self.current).unwrap().clone() {
                self.current += 1;
                let initializer = self.expression();
                self.consume(lexer::Token::Semicolon);
                return Stmt::Variable {
                    name: lexer::Token::Identifier(name),
                    initializer: Box::new(initializer),
                };
            }
            return Stmt::Variable {
                name: lexer::Token::Identifier(name),
                initializer: Box::new(ast::Expr::L(ast::Literal::Nil(()))),
            };
        } else {
            panic!()
        }
    }

    pub fn expression(&mut self) -> ast::Expr {
        self.assignment()
    }

    pub fn assignment(&mut self) -> ast::Expr {
        let expr = self.equality();
        if self.tokens.get(self.current).unwrap() == &lexer::Token::Assign {
            self.current += 1;
            let value = self.assignment();

            if let ast::Expr::Variable { name } = expr {
                return ast::Expr::Assign {
                    name,
                    value: Box::new(value),
                };
            }
            panic!()
        }
        expr
    }

    pub fn equality(&mut self) -> ast::Expr {
        let mut expr = self.comparsion();

        while self.current < self.tokens.len()
            && (self.tokens.get(self.current).unwrap().clone() == lexer::Token::O(Operator::Equal)
                || self.tokens.get(self.current).unwrap().clone()
                    == lexer::Token::O(Operator::NotEqual))
        {
            self.current += 1;
            if let lexer::Token::O(operator) = self.previous() {
                let right = self.comparsion();
                expr = ast::Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            } else {
                panic!()
            }
        }
        expr
    }

    pub fn comparsion(&mut self) -> ast::Expr {
        let mut expr = self.addition();

        while self.current < self.tokens.len()
            && (self.tokens.get(self.current).unwrap().clone()
                == lexer::Token::O(Operator::GreaterThan)
                || self.tokens.get(self.current).unwrap().clone()
                    == lexer::Token::O(Operator::GreaterThanOrEqual)
                || self.tokens.get(self.current).unwrap().clone()
                    == lexer::Token::O(Operator::LessThan)
                || self.tokens.get(self.current).unwrap().clone()
                    == lexer::Token::O(Operator::LessThanOrEqual))
        {
            self.current += 1;
            if let lexer::Token::O(operator) = self.previous() {
                let right = self.addition();
                expr = ast::Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            } else {
                panic!()
            }
        }
        expr
    }

    pub fn addition(&mut self) -> ast::Expr {
        let mut expr = self.multiplication();

        while self.current < self.tokens.len()
            && (self.tokens.get(self.current).unwrap().clone() == lexer::Token::O(Operator::Minus)
                || self.tokens.get(self.current).unwrap().clone()
                    == lexer::Token::O(Operator::Plus))
        {
            self.current += 1;
            if let lexer::Token::O(operator) = self.previous() {
                let right = self.multiplication();
                expr = ast::Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            } else {
                panic!()
            }
        }
        expr
    }

    pub fn multiplication(&mut self) -> ast::Expr {
        let mut expr = self.unary();

        while self.current < self.tokens.len()
            && (self.tokens.get(self.current).unwrap().clone() == lexer::Token::O(Operator::Slash)
                || self.tokens.get(self.current).unwrap().clone()
                    == lexer::Token::O(Operator::Star))
        {
            self.current += 1;
            if let lexer::Token::O(operator) = self.previous() {
                let right = self.unary();
                expr = ast::Expr::Binary {
                    left: Box::new(expr),
                    operator,
                    right: Box::new(right),
                }
            } else {
                panic!()
            }
        }
        expr
    }

    pub fn unary(&mut self) -> ast::Expr {
        if self.tokens.get(self.current).unwrap().clone() == lexer::Token::O(Operator::Not)
            || self.tokens.get(self.current).unwrap().clone() == lexer::Token::O(Operator::Minus)
        {
            self.current += 1;
            if let lexer::Token::O(operator) = self.previous() {
                let right = self.unary();
                ast::Expr::Unary {
                    operator,
                    right: Box::new(right),
                }
            } else {
                panic!()
            }
        } else {
            self.primary()
        }
    }

    pub fn primary(&mut self) -> ast::Expr {
        let cur = self.tokens.get(self.current).unwrap().clone();
        self.current += 1;
        if let lexer::Token::Identifier(_) = cur {
            ast::Expr::Variable { name: cur.clone() }
        } else if cur == lexer::Token::LeftParen {
            let expr = self.expression();
            self.consume(lexer::Token::RightParen);
            ast::Expr::Grouping {
                expression: Box::new(expr),
            }
        } else {
            match cur {
                lexer::Token::Bool(b) => ast::Expr::L(ast::Literal::Bool(b)),
                lexer::Token::Nil(n) => ast::Expr::L(ast::Literal::Nil(n)),
                lexer::Token::Float(f) => ast::Expr::L(ast::Literal::Float(f)),
                lexer::Token::String(s) => ast::Expr::L(ast::Literal::String(s)),
                _ => panic!("{:?}", cur),
            }
        }
    }

    pub fn previous(&mut self) -> lexer::Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    pub fn consume(&mut self, token: lexer::Token) {
        if self.tokens.get(self.current).unwrap().clone() == token {
            self.current += 1;
        } else {
            panic!()
        }
    }

    pub fn print_current(&self) {
        println!("{:?}", self.tokens.get(self.current))
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::lexer;
    use crate::parser::parser::Parser;

    #[test]
    fn parse_test() {
        let input: Vec<char> = "nil".chars().collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.expression();
        println!("{:#?}", e);
    }

    #[test]
    fn parse_print() {
        let input: Vec<char> = "print 5;".chars().collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse_stmts();
        println!("{:#?}", e);
    }

    #[test]
    fn parse_var() {
        let input: Vec<char> = "var x = 5;".chars().collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse_stmts();
        println!("{:#?}", e);
    }

    #[test]
    fn parse_parse_block() {
        let input: Vec<char> = "{var x = 5;} {var y = 10;} {print y;}".chars().collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse_stmts();
        println!("{:#?}", e);
    }
}
