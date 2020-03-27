use crate::lexer;
use crate::lexer::{Operator, Token};
use crate::parser::ast;
use crate::parser::ast::Stmt;

pub struct Parser {
    tokens: Vec<lexer::Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<lexer::Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<ast::Stmt> {
        let mut statements: Vec<ast::Stmt> = Vec::new();
        while self.current < self.tokens.len() {
            statements.push(self.delaration());
        }
        statements
    }

    pub fn statement(&mut self) -> ast::Stmt {
        if self.tokens.get(self.current).unwrap().clone() == lexer::Token::For {
            self.current += 1;
            return self.for_stmt();
        } else if self.tokens.get(self.current).unwrap().clone() == lexer::Token::If {
            self.current += 1;
            return self.if_stmt();
        } else if self.tokens.get(self.current).unwrap().clone() == lexer::Token::Print {
            self.current += 1;
            return self.print();
        } else if self.tokens.get(self.current).unwrap().clone() == lexer::Token::While {
            self.current += 1;
            return self.while_stmt();
        } else if self.tokens.get(self.current).unwrap().clone() == lexer::Token::LeftBrace {
            self.current += 1;
            return Stmt::Block(self.block());
        }
        self.expr_stmt()
    }

    pub fn expr_stmt(&mut self) -> ast::Stmt {
        let expr = self.expression();
        self.consume(lexer::Token::Semicolon);
        ast::Stmt::Expr(Box::new(expr))
    }

    pub fn for_stmt(&mut self) -> ast::Stmt {
        self.consume(Token::LeftParen);
        let mut initializer = None;
        let mut condition = None;
        let mut increment = None;
        if let Some(token) = self.tokens.get(self.current) {
            if token == &lexer::Token::Semicolon {
                self.current += 1;
            } else if token == &lexer::Token::Var {
                self.current += 1;
                initializer = Some(self.var_decl());
            } else {
                initializer = Some(self.expr_stmt());
            }

            if let Some(token) = self.tokens.get(self.current) {
                if token != &lexer::Token::Semicolon {
                    condition = Some(self.expression());
                }
            }
            self.consume(lexer::Token::Semicolon);

            if let Some(token) = self.tokens.get(self.current) {
                if token != &lexer::Token::RightParen {
                    increment = Some(self.expression());
                }
            }
            self.consume(lexer::Token::RightParen);
            let mut body = self.statement();

            if let Some(increment) = increment {
                body = Stmt::Block(vec![body, Stmt::Expr(Box::new(increment))])
            }

            if let Some(condition) = condition {
                body = Stmt::While {
                    condition: Box::new(condition),
                    body: Box::new(body),
                }
            }

            if let Some(initializer) = initializer {
                body = Stmt::Block(vec![initializer, body])
            }
            return body;
        } else {
            panic!()
        }
    }


    pub fn if_stmt(&mut self) -> ast::Stmt {
        self.consume(Token::LeftParen);
        let condition = self.expression();
        self.consume(Token::RightParen);

        let then_branch = self.statement();
        let mut else_branch = None;
        if let Some(token) = self.tokens.get(self.current) {
            if token == &lexer::Token::Else {
                self.current += 1;
                else_branch = Some(Box::new(self.statement()));
            }
        }
        ast::Stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch,
        }
    }

    pub fn while_stmt(&mut self) -> ast::Stmt {
        self.consume(Token::LeftParen);
        let condition = self.expression();
        self.consume(Token::RightParen);

        let body = self.statement();
        ast::Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        }
    }

    pub fn print(&mut self) -> ast::Stmt {
        let value = self.expression();
        self.consume(lexer::Token::Semicolon);
        ast::Stmt::Print(Box::new(value))
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
        if self.tokens.get(self.current).unwrap().clone() == lexer::Token::Fun {
            self.current += 1;
            return self.func_decl("function");
        }
        if self.tokens.get(self.current).unwrap().clone() == lexer::Token::Var {
            self.current += 1;
            return self.var_decl();
        }
        self.statement()
    }

    pub fn func_decl(&mut self, kind: &str) -> ast::Stmt {
        if let lexer::Token::Identifier(name) = self.tokens.get(self.current).unwrap().clone() {
            self.current += 1;
            self.consume(lexer::Token::LeftParen);
            let mut params = Vec::new();
            if self.tokens.get(self.current).unwrap().clone() != lexer::Token::RightParen {
                if params.len() >= 255 {
                    panic!("too many args");
                }
                params.push(self.tokens.get(self.current).unwrap().clone());
                self.current += 1;
                while self.tokens.get(self.current).unwrap().clone() == lexer::Token::Comma {
                    self.current += 1;
                    params.push(self.tokens.get(self.current).unwrap().clone());
                    self.current += 1;
                }
            }
            self.consume(lexer::Token::RightParen);
            let body = self.statement();
            return ast::Stmt::Function { name: lexer::Token::Identifier(name), parameters: Some(params), body: Box::new(body), ret: None }
        }
        panic!("{:?}", self.tokens.get(self.current).unwrap().clone())
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
        let expr = self.or();
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

    pub fn or(&mut self) -> ast::Expr {
        let mut expr = self.and();
        while self.current < self.tokens.len()
            && self.tokens.get(self.current).unwrap().clone() == lexer::Token::O(Operator::Or)
        {
            self.current += 1;
            let right = self.and();
            expr = ast::Expr::Logical {
                left: Box::new(expr),
                operator: Operator::Or,
                right: Box::new(right),
            }
        }
        expr
    }

    pub fn and(&mut self) -> ast::Expr {
        let mut expr = self.equality();
        while self.current < self.tokens.len()
            && self.tokens.get(self.current).unwrap().clone() == lexer::Token::O(Operator::And)
        {
            self.current += 1;
            let right = self.and();
            expr = ast::Expr::Logical {
                left: Box::new(expr),
                operator: Operator::And,
                right: Box::new(right),
            }
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
            self.call()
        }
    }

    pub fn call(&mut self) -> ast::Expr {
        let mut expr = self.primary();

        loop {
            if self.tokens.get(self.current).unwrap().clone() == lexer::Token::LeftParen {
                self.current += 1;
                expr = self.finish_call(expr)
            } else {
                break;
            }
        }
        expr
    }

    pub fn finish_call(&mut self, callee: ast::Expr) -> ast::Expr {
        let mut arguments = Vec::new();
        if self.tokens.get(self.current).unwrap().clone() != lexer::Token::RightParen {
            if arguments.len() >= 255 {
                panic!("too many args");
            }
            arguments.push(self.expression());
            while self.tokens.get(self.current).unwrap().clone() == lexer::Token::Comma {
                self.current += 1;
                arguments.push(self.expression());
            }
        }
        self.consume(lexer::Token::RightParen);
        ast::Expr::Call {
            callee: Box::new(callee),
            arguments,
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
            panic!(
                "token {:?} current {:?}",
                token,
                self.tokens.get(self.current)
            )
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
        let input: Vec<char> = "nil;".chars().collect();
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
        let e = p.parse();
        println!("{:#?}", e);
    }

    #[test]
    fn parse_var() {
        let input: Vec<char> = "var x = 5;".chars().collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        println!("{:#?}", e);
    }

    #[test]
    fn parse_block() {
        let input: Vec<char> = "{var x = 5;} {var y = 10;} {print y;}".chars().collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        println!("{:#?}", e);
    }

    #[test]
    fn parse_if() {
        let input: Vec<char> = "if (x < 5) { print x; } else { print 5; }"
            .chars()
            .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        println!("{:#?}", e);
    }

    #[test]
    fn parse_logical() {
        let input: Vec<char> = "a and b or c;".chars().collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        println!("{:#?}", e);
    }

    #[test]
    fn parse_assign() {
        let input: Vec<char> = r#"
            var a = 5;
            print a;
            a = a + 1;
            print a + a;
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        println!("{:#?}", e);
    }

    #[test]
    fn parse_for() {
        let input: Vec<char> = r#"
            for (var i = 0; i < 10; i = i + 1) {
                var j = 2;
                print i;
                print j*i;
            }
        "#
        .chars()
        .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        println!("{:#?}", e);
    }


    #[test]
    fn clock() {
        let input: Vec<char> = r#"
            clock();
        "#
            .chars()
            .collect();
        let tokens = lexer().parse(&input).unwrap();
        let mut p = Parser::new(tokens);
        let e = p.parse();
        println!("{:#?}", e);
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
        println!("{:#?}", e);
    }
}
