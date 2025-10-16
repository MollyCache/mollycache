use crate::db::table::core::column::ColumnDefinition;
use crate::interpreter::{
    ast::helpers::common::get_table_name,
    ast::helpers::token::{expect_token_type, token_to_data_type},
    ast::{AlterTableAction, AlterTableStatement, SqlStatement, parser::Parser},
    tokenizer::token::TokenTypes,
};

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    expect_token_type(parser, TokenTypes::Table)?;
    parser.advance()?;
    let (table_name, table_alias) = get_table_name(parser)?;
    if table_alias != "" {
        return Err("Table aliases in CREATE TABLE statement not allowed".to_string());
    }
    let action = get_action(parser)?;
    parser.advance()?;
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return Ok(SqlStatement::AlterTable(AlterTableStatement {
        table_name: table_name,
        action: action,
    }));
}

fn get_action(parser: &mut Parser) -> Result<AlterTableAction, String> {
    return match parser.current_token()?.token_type {
        TokenTypes::Rename => {
            parser.advance()?;
            let action = match parser.current_token()?.token_type {
                TokenTypes::Column => {
                    parser.advance()?;
                    expect_token_type(parser, TokenTypes::Identifier)?;
                    let old_column_name = parser.current_token()?.value.to_string();
                    parser.advance()?;
                    expect_token_type(parser, TokenTypes::To)?;
                    parser.advance()?;
                    expect_token_type(parser, TokenTypes::Identifier)?;
                    let new_column_name = parser.current_token()?.value.to_string();
                    AlterTableAction::RenameColumn {
                        old_column_name,
                        new_column_name,
                    }
                }
                TokenTypes::To => {
                    parser.advance()?;
                    expect_token_type(parser, TokenTypes::Identifier)?;
                    let new_table_name = parser.current_token()?.value.to_string();
                    AlterTableAction::RenameTable { new_table_name }
                }
                _ => return Err(parser.format_error()),
            };
            Ok(action)
        }
        TokenTypes::Add => {
            parser.advance()?;
            expect_token_type(parser, TokenTypes::Column)?;
            parser.advance()?;
            expect_token_type(parser, TokenTypes::Identifier)?;
            let name = parser.current_token()?.value.to_string();
            parser.advance()?;
            let data_type = token_to_data_type(parser)?;
            Ok(AlterTableAction::AddColumn {
                column_def: ColumnDefinition {
                    name,
                    data_type,
                    constraints: vec![],
                },
            })
        }
        TokenTypes::Drop => {
            parser.advance()?;
            expect_token_type(parser, TokenTypes::Column)?;
            parser.advance()?;
            expect_token_type(parser, TokenTypes::Identifier)?;
            let column_name = parser.current_token()?.value.to_string();
            Ok(AlterTableAction::DropColumn { column_name })
        }
        _ => return Err(parser.format_error()),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::value::DataType;
    use crate::interpreter::ast::test_utils::token;

    #[test]
    fn alter_table_statement_with_all_tokens_is_generated_correctly() {
        // ALTER TABLE users RENAME COLUMN name TO new_name;
        let tokens = vec![
            token(TokenTypes::Alter, "ALTER"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Rename, "RENAME"),
            token(TokenTypes::Column, "COLUMN"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::To, "TO"),
            token(TokenTypes::Identifier, "new_name"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::AlterTable(AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::RenameColumn {
                old_column_name: "name".to_string(),
                new_column_name: "new_name".to_string(),
            },
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn alter_table_statement_with_rename_table_is_generated_correctly() {
        // ALTER TABLE users RENAME TO new_name;
        let tokens = vec![
            token(TokenTypes::Alter, "ALTER"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Rename, "RENAME"),
            token(TokenTypes::To, "TO"),
            token(TokenTypes::Identifier, "new_name"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::AlterTable(AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::RenameTable {
                new_table_name: "new_name".to_string(),
            },
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn alter_table_statement_with_add_column_is_generated_correctly() {
        // ALTER TABLE users ADD COLUMN name BLOB;
        let tokens = vec![
            token(TokenTypes::Alter, "ALTER"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Add, "ADD"),
            token(TokenTypes::Column, "COLUMN"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Blob, "BLOB"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::AlterTable(AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::AddColumn {
                column_def: ColumnDefinition {
                    name: "name".to_string(),
                    data_type: DataType::Blob,
                    constraints: vec![],
                },
            },
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn alter_table_statement_with_drop_column_is_generated_correctly() {
        // ALTER TABLE users DROP COLUMN name;
        let tokens = vec![
            token(TokenTypes::Alter, "ALTER"),
            token(TokenTypes::Table, "TABLE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Drop, "DROP"),
            token(TokenTypes::Column, "COLUMN"),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::AlterTable(AlterTableStatement {
            table_name: "users".to_string(),
            action: AlterTableAction::DropColumn {
                column_name: "name".to_string(),
            },
        });
        assert_eq!(expected, statement);
    }
}
