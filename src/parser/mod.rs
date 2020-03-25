pub mod ast;
pub mod interpreter;
pub mod parser;
mod ast_printer;

#[cfg(test)]
mod tests {
    use crate::environment::Environment;
    use crate::lexer;
    use crate::parser::interpreter::Interpreter;
    use crate::parser::parser;

    #[test]
    pub fn test_stmts() {
        let input: Vec<char> = "\
            print \"one\"; \
            print true; \
            print 2 + 1;\
        "
        .chars()
        .collect();
        let tokens = lexer::lexer().parse(&input).unwrap();
        let mut p = parser::Parser::new(tokens);
        (Interpreter {
            environment: Default::default(),
        })
        .interpret(&p.parse_stmts());
    }
}
