use crate::interpreter::ast::{parser::Parser, SqlStatement, BeginStatement, RollbackStatement, SavepointStatement, ReleaseStatement};
use crate::interpreter::tokenizer::token::TokenTypes;
use crate::interpreter::ast::helpers::token::expect_token_type;

pub fn build_begin(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    if expect_token_type(parser, TokenTypes::Deferred).is_ok() || expect_token_type(parser, TokenTypes::SemiColon).is_ok() {
        return Ok(SqlStatement::BeginTransaction(BeginStatement::Deferred));
    } else if expect_token_type(parser, TokenTypes::Exclusive).is_ok() {
        return Ok(SqlStatement::BeginTransaction(BeginStatement::Exclusive));
    }
    else if expect_token_type(parser, TokenTypes::Immediate).is_ok() {
        return Ok(SqlStatement::BeginTransaction(BeginStatement::Immediate));
    }
    else {
        return Err(parser.format_error());
    }
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