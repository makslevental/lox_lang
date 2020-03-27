pub mod ast;
pub mod parser;

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::lexer;
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
        Interpreter::new().interpret(&p.parse());
    }
}
