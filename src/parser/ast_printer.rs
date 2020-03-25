use crate::parser::ast::{Expr, ExprVisitor, Literal, ExprData};

pub struct AstPrinter;

impl AstPrinter {
    fn print(&mut self, expr: &Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&mut self, name: &str, exprs: &[&Expr]) -> String {
        let mut builder = String::new();
        builder.push('(');
        builder.push_str(name);

        for expr in exprs {
            builder.push(' ');
            builder.push_str(expr.accept(self).as_str());
        }

        builder.push(')');
        builder
    }
}

impl ExprVisitor for AstPrinter {
    type Result = String;

    fn visit_literal(&mut self, expr: &Literal) -> String {
        match expr {
            Literal::Float(l) => format!("{:?}", l),
            Literal::Bool(l) => format!("{:?}", l),
            Literal::String(l) => format!("{:?}", l),
            Literal::Nil(_) => "nil".to_string(),
        }
    }

    fn visit_unary(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Unary { operator, right } = expr {
            self.parenthesize(format!("{}", operator).as_str(), &[right])
        } else {
            panic!()
        }
    }

    fn visit_binary(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Binary {
            left,
            operator,
            right,
        } = expr
        {
            self.parenthesize(format!("{}", operator).as_str(), &[left, right])
        } else {
            panic!()
        }
    }

    fn visit_logical(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Logical {
            left,
            operator,
            right,
        } = expr
        {
            self.parenthesize(format!("{}", operator).as_str(), &[left, right])
        } else {
            panic!()
        }
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Grouping { expression } = expr {
            self.parenthesize("group", &[expression])
        } else {
            panic!()
        }
    }

    fn visit_assign(&mut self, expr: &Expr) -> Self::Result {
        if let Expr::Assign {
            name: identifier,
            value: expr,
        } = expr
        {
            self.parenthesize(format!("{} =", identifier).as_str(), &[expr])
        } else {
            panic!()
        }
    }

    fn visit_variable(&mut self, expr: &Expr) -> Self::Result {
        unimplemented!()
    }
}
#[cfg(test)]
mod tests {
    use crate::environment::Environment;
    use crate::lexer::{Operator, Token};
    use crate::parser::ast::{Expr, Literal, Stmt};
    use crate::parser::interpreter::{Interpreter};
    use super::AstPrinter;

    #[test]
    fn print() {
        let expr = Box::new(Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Operator::Minus,
                right: Box::new(Expr::L(Literal::Float(123.0))),
            }),
            operator: Operator::Star,
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::L(Literal::Float(45.67))),
            }),
        });

        println!("{}", (AstPrinter {}).print(&expr));
    }

    #[test]
    fn assign() {
        let x = String::from("x");
        let expr = Box::new(Expr::Assign {
            name: Token::Identifier(x),
            value: Box::new(Expr::L(Literal::Float(45.67))),
        });

        println!("{}", (AstPrinter {}).print(&expr));
    }
}