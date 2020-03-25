use crate::lexer;
use crate::parser::ast::Literal;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Environment {
    values: HashMap<String, Literal>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn define(&mut self, name: &str, value: Literal) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &lexer::Token) -> Literal {
        if let lexer::Token::Identifier(name) = name {
            self.values.get(name.as_str()).unwrap().clone()
        } else if let Some(ref env) = self.enclosing {
            env.get(name)
        } else {
            panic!()
        }
    }

    pub fn assign(&mut self, name: &str, value: Literal) {
        if self.values.contains_key(name) {
            self.values.insert(name.to_owned(), value);
        } else if let Some(ref mut env) = self.enclosing {
            env.assign(name, value)
        } else {
            panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::environment::Environment;
    use crate::lexer::Token;
    use crate::parser::ast::Literal;

    #[test]
    fn test() {
        let mut e = Environment {
            values: Default::default(),
            enclosing: None,
        };
        e.define("bob", Literal::Float(1.0));
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
