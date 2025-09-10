use crate::interpreter::ast::{parser::Parser, SqlStatement, BeginStatement, RollbackStatement, SavepointStatement, ReleaseStatement};
use crate::interpreter::tokenizer::token::TokenTypes;
use crate::interpreter::ast::helpers::token::expect_token_type;

pub fn build_begin(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let statement = if expect_token_type(parser, TokenTypes::Deferred).is_ok() || expect_token_type(parser, TokenTypes::SemiColon).is_ok() {
        SqlStatement::BeginTransaction(BeginStatement::Deferred)
    } else if expect_token_type(parser, TokenTypes::Exclusive).is_ok() {
        SqlStatement::BeginTransaction(BeginStatement::Exclusive)
    }
    else if expect_token_type(parser, TokenTypes::Immediate).is_ok() {
        SqlStatement::BeginTransaction(BeginStatement::Immediate)
    }
    else {
        return Err(parser.format_error());
    };  
    if parser.current_token()?.token_type != TokenTypes::SemiColon {
        parser.advance()?;
        expect_token_type(parser, TokenTypes::SemiColon)?;
    }
    return Ok(statement);
}

pub fn build_commit(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return Ok(SqlStatement::Commit);
}

pub fn build_rollback(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let name = if expect_token_type(parser, TokenTypes::To).is_ok() {
        parser.advance()?;
        expect_token_type(parser, TokenTypes::Savepoint)?;
        parser.advance()?;
        expect_token_type(parser, TokenTypes::Identifier)?;
        Some(parser.current_token()?.value.to_string())
    } else {
        None
    };
    parser.advance()?;
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return Ok(SqlStatement::Rollback(RollbackStatement {
        savepoint_name: name,
    }));
}

pub fn build_savepoint(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    expect_token_type(parser, TokenTypes::Identifier)?;
    let savepoint_name = parser.current_token()?.value.to_string();
    parser.advance()?;
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return Ok(SqlStatement::Savepoint(SavepointStatement {
        savepoint_name: savepoint_name,
    }));
}

pub fn build_release(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    expect_token_type(parser, TokenTypes::Savepoint)?;
    parser.advance()?;
    expect_token_type(parser, TokenTypes::Identifier)?;
    let savepoint_name = parser.current_token()?.value.to_string();
    parser.advance()?;
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return Ok(SqlStatement::Release(ReleaseStatement {
        savepoint_name: savepoint_name,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::test_utils::token;
        
    #[test]
    fn build_begin_with_all_tokens_is_generated_correctly() {
        // BEGIN DEFERRED; BEGIN EXCLUSIVE; BEGIN IMMEDIATE; BEGIN;
        let begin_tokens = vec! [
            token(TokenTypes::Begin, "BEGIN"),
            token(TokenTypes::Deferred, "DEFERRED"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Begin, "BEGIN"),
            token(TokenTypes::Exclusive, "EXCLUSIVE"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Begin, "BEGIN"),
            token(TokenTypes::Immediate, "IMMEDIATE"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Begin, "BEGIN"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let expected = vec![
            Some(Ok(SqlStatement::BeginTransaction(BeginStatement::Deferred))), 
            Some(Ok(SqlStatement::BeginTransaction(BeginStatement::Exclusive))), 
            Some(Ok(SqlStatement::BeginTransaction(BeginStatement::Immediate))),
            Some(Ok(SqlStatement::BeginTransaction(BeginStatement::Deferred)))
        ];
        let mut  parser = Parser::new(begin_tokens);
        for i in 0..3 {
            let result = parser.next_statement();
            assert_eq!(expected[i], result);
            let _ = parser.advance_past_semicolon();
        }
    }
}