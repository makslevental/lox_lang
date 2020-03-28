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
pub struct SymbolTable {
    pub enclosing: Option<Rc<RefCell<SymbolTable>>>,
    pub values: Rc<RefCell<HashMap<String, Object>>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        return Self {
            enclosing: None,
            values: Rc::new(RefCell::new(Default::default())),
        };
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.borrow_mut().insert(name.to_owned(), value);
    }

    pub fn exists(&mut self, name: &str) -> bool {
        self.values.borrow().contains_key(name)
    }

    pub fn get(&self, name: &str) -> Object {
        if self.values.borrow().contains_key(name) {
            self.values.borrow().get(name).unwrap().clone()
        } else if let Some(ref env) = self.enclosing {
            env.borrow().get(name)
        } else {
            panic!("{:?} {:#?}", name, self)
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

    pub fn deep_copy(&self) -> Self {
        let mut values = HashMap::new();
        for (name, object) in self.values.borrow().iter() {
            values.insert(name.to_owned(), object.clone());
        }
        SymbolTable {
            enclosing: self.enclosing.clone(),
            values: Rc::new(RefCell::new(values)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token;
    use crate::parser::ast::Literal;
    use crate::symbol_table::{Object, SymbolTable};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test() {
        let mut e = SymbolTable {
            values: Default::default(),
            enclosing: None,
        };
        e.define("bob", Object::L(Literal::Float(1.0)));
        println!("{:?}", e.get("bob"));
    }

    #[test]
    fn env() {
        let mut e = SymbolTable {
            values: Default::default(),
            enclosing: None,
        };
        println!("{:?}", e);
    }
    #[test]
    fn inner() {
        let mut outer = Rc::new(RefCell::new(SymbolTable {
            values: Default::default(),
            enclosing: None,
        }));

        outer
            .borrow_mut()
            .define("x", Object::L(Literal::String("inner".to_string())));
        println!("{:#?}", outer);

        let mut inner = SymbolTable {
            values: Default::default(),
            enclosing: Some(outer.clone()),
        };
        println!("{:#?}", inner);
        inner.assign("x", Object::L(Literal::String("outer".to_string())));
        println!("{:#?}", inner);
        println!("{:#?}", outer);

        let mut copy = inner.deep_copy();
        copy.assign("x", Object::L(Literal::String("copy".to_string())));
        println!("{:#?}", copy);
        println!("{:#?}", inner);
        println!("{:#?}", outer);
    }
}
