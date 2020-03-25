use crate::lexer;

pub trait StmtData {
    fn accept<V: StmtVisitor>(&self, visitor: &mut V);
}

pub trait StmtVisitor {
    fn visit_expr_stmt(&mut self, stmt: &Stmt);
    fn visit_print_stmt(&mut self, stmt: &Stmt);
    fn visit_var_stmt(&mut self, stmt: &Stmt);
    fn visit_block(&mut self, stmt: &Stmt);
    fn visit_if(&mut self, stmt: &Stmt);
    fn visit_while(&mut self, stmt: &Stmt);
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Box<Expr>),
    Print(Box<Expr>),
    Variable {
        name: lexer::Token,
        initializer: Box<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    }
}

impl StmtData for Stmt {
    fn accept<V: StmtVisitor>(&self, visitor: &mut V) {
        match self {
            s @ Stmt::Expr(_) => visitor.visit_expr_stmt(s),
            s @ Stmt::Print(_) => visitor.visit_print_stmt(s),
            s @ Stmt::Variable { .. } => visitor.visit_var_stmt(s),
            s @ Stmt::Block(_) => visitor.visit_block(s),
            s @ Stmt::If { .. } => visitor.visit_if(s),
            s @ Stmt::While { .. } => visitor.visit_while(s),
        }
    }
}

pub trait ExprData {
    fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Result;
}

pub trait ExprVisitor {
    type Result;

    fn visit_expr(&mut self, expr: &Expr) -> Self::Result {
        match expr {
            Expr::L(l) => self.visit_literal(l),
            e @ Expr::Unary { .. } => self.visit_unary(e),
            e @ Expr::Binary { .. } => self.visit_binary(e),
            e @ Expr::Logical { .. } => self.visit_logical(e),
            e @ Expr::Grouping { .. } => self.visit_grouping(e),
            e @ Expr::Assign { .. } => self.visit_assign(e),
            e @ Expr::Variable { .. } => self.visit_variable(e),
        }
    }
    fn visit_literal(&mut self, expr: &Literal) -> Self::Result;
    fn visit_unary(&mut self, expr: &Expr) -> Self::Result;
    fn visit_binary(&mut self, expr: &Expr) -> Self::Result;
    fn visit_logical(&mut self, expr: &Expr) -> Self::Result;
    fn visit_grouping(&mut self, expr: &Expr) -> Self::Result;
    fn visit_assign(&mut self, expr: &Expr) -> Self::Result;
    fn visit_variable(&mut self, expr: &Expr) -> Self::Result;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Float(f64),
    Bool(bool),
    String(String),
    Nil(()),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    L(Literal),
    Unary {
        operator: lexer::Operator,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: lexer::Operator,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: lexer::Operator,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Assign {
        name: lexer::Token,
        value: Box<Expr>,
    },
    Variable {
        name: lexer::Token,
    },
}

impl ExprData for Expr {
    fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Result {
        match self {
            Expr::L(l) => visitor.visit_literal(l),
            e @ Expr::Unary { .. } => visitor.visit_unary(e),
            e @ Expr::Binary { .. } => visitor.visit_binary(e),
            e @ Expr::Logical { .. } => visitor.visit_logical(e),
            e @ Expr::Grouping { .. } => visitor.visit_grouping(e),
            e @ Expr::Assign { .. } => visitor.visit_assign(e),
            e @ Expr::Variable { .. } => visitor.visit_variable(e),
        }
    }
}
