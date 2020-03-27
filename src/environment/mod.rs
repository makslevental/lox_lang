use crate::interpreter::callable::Callable;
use crate::lexer;
use crate::parser::ast::Literal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Object {
    L(Literal),
    C(Rc<dyn Callable<Result = Object>>),
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    pub values: Rc<RefCell<HashMap<String, Object>>>,
}

impl Environment {
    pub fn new() -> Self {
        return Self {
            enclosing: None,
            values: Rc::new(RefCell::new(Default::default())),
        };
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.borrow_mut().insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &lexer::Token) -> Object {
        if let lexer::Token::Identifier(name) = name {
            if self.values.borrow().contains_key(name) {
                self.values.borrow().get(name.as_str()).unwrap().clone()
            } else if let Some(ref env) = self.enclosing {
                env.borrow()
                    .get(&lexer::Token::Identifier(name.to_string()))
            } else {
                panic!("{:?} {:#?}", name, self)
            }
        } else {
            panic!()
        }
    }

    pub fn assign(&mut self, name: &str, value: Object) {
        if self.values.borrow().contains_key(name) {
            self.values.borrow_mut().insert(name.to_owned(), value);
        } else if let Some(ref mut env) = self.enclosing {
            env.borrow_mut().assign(name, value)
        } else {
            panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::environment::{Environment, Object};
    use crate::lexer::Token;
    use crate::parser::ast::Literal;

    #[test]
    fn test() {
        let mut e = Environment {
            values: Default::default(),
            enclosing: None,
        };
        e.define("bob", Object::L(Literal::Float(1.0)));
        println!("{:?}", e.get(&Token::Identifier("bob".to_string())));
    }

    #[test]
    fn env() {
        let mut e = Environment {
            values: Default::default(),
            enclosing: None,
        };
        println!("{:?}", e);
    }
}
