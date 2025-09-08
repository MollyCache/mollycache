use crate::interpreter::{
    ast::{
        parser::Parser, SqlStatement, DropTableStatement, ExistenceCheck,
        helpers::common::{expect_token_type, get_table_name, exists_clause}
    },
    tokenizer::token::TokenTypes
};

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    expect_token_type(parser, TokenTypes::Table)?;
    parser.advance()?;

    let existence_check = exists_clause(parser, ExistenceCheck::IfExists)?;
    let table_name = get_table_name(parser)?;
    return Ok(SqlStatement::DropTable(DropTableStatement {
        table_name: table_name,
        existence_check: existence_check,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::test_utils::token;

    #[test]
    fn drop_statement_with_all_tokens_is_generated_correctly() {
        // DROP TABLE users;
        let tokens = vec![
            token(TokenTypes::Drop, "DROP"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::DropTable(DropTableStatement {
            table_name: "users".to_string(),
            existence_check: None,
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn drop_statement_with_if_exists_clause_is_generated_correctly() {
        // DROP TABLE IF EXISTS users;
        let tokens = vec![
            token(TokenTypes::Drop, "DROP"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::If, "IF"),
            token(TokenTypes::Exists, "EXISTS"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::DropTable(DropTableStatement {
            table_name: "users".to_string(),
            existence_check: Some(ExistenceCheck::IfExists),
        });
        assert_eq!(expected, statement);
    }

}