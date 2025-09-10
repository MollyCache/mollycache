use crate::interpreter::ast::{parser::Parser, SqlStatement, BeginStatement};
use crate::interpreter::tokenizer::token::TokenTypes;
use crate::interpreter::ast::helpers::token::expect_token_type;

pub fn build_begin(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    if (expect_token_type(parser, TokenTypes::Deferred).is_ok() || expect_token_type(parser, TokenTypes::SemiColon).is_ok()) {
        return Ok(SqlStatement::BeginTransaction(BeginStatement::Deferred));
    } else if (expect_token_type(parser, TokenTypes::Exclusive).is_ok()) {
        return Ok(SqlStatement::BeginTransaction(BeginStatement::Exclusive));
    }
    else if (expect_token_type(parser, TokenTypes::Immediate).is_ok()) {
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
    todo!()
}

pub fn build_savepoint(parser: &mut Parser) -> Result<SqlStatement, String> {
    todo!()
}

pub fn build_release(parser: &mut Parser) -> Result<SqlStatement, String> {
    todo!()
}