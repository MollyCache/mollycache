use crate::db::table::core::column::ColumnDefinition;
use crate::db::table::core::column::ColumnStack;
use crate::db::table::core::row::Row;
use crate::db::table::core::row::RowStack;
use crate::db::table::core::value::Value;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: ColumnStack,
    rows: Vec<RowStack>,
}

impl Index<usize> for Table {
    type Output = Row;

    fn index(&self, index: usize) -> &Self::Output {
        self.rows[index].stack.last().unwrap()
    }
}

impl IndexMut<usize> for Table {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.rows[index].stack.last_mut().unwrap()
    }
}

impl Table {
    pub fn new(name: String, columns: Vec<ColumnDefinition>) -> Self {
        Self {
            name,
            columns: ColumnStack::new(columns),
            rows: vec![],
        }
    }

    pub fn get(&self, i: usize) -> Option<&Row> {
        self.rows.get(i)?.stack.last()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().map(|s| s.stack.last().unwrap())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Row> {
        self.rows.iter_mut().map(|s| s.stack.last_mut().unwrap())
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn swap(&mut self, a: usize, b: usize) -> () {
        self.rows.swap(a, b);
    }

    pub fn get_rows_clone(&self) -> Vec<Row> {
        self.rows
            .iter()
            .map(|s| s.stack.last().unwrap().clone())
            .collect()
    }

    pub fn get_rows(&self) -> Vec<&Row> {
        self.rows.iter().map(|s| s.stack.last().unwrap()).collect()
    }

    pub fn get_rows_mut(&mut self) -> Vec<&mut Row> {
        self.rows
            .iter_mut()
            .map(|s| s.stack.last_mut().unwrap())
            .collect()
    }

    pub fn set_rows(&mut self, rows: Vec<Row>) {
        self.rows = rows.into_iter().map(|r| RowStack::new(r)).collect();
    }

    pub fn push(&mut self, row: Row) {
        self.rows.push(RowStack::new(row));
    }

    pub fn pop(&mut self) -> Option<Row> {
        self.rows.pop().and_then(|mut value| value.stack.pop())
    }

    pub fn commit_transaction(&mut self, affected_row_indices: &Vec<usize>) -> Result<(), String> {
        // Keep only the top of the each row stack.
        for index in affected_row_indices {
            if let Some(row_stack) = self.rows.get_mut(*index) {
                row_stack.stack = vec![row_stack.stack.last().unwrap().clone()];
            } else {
                return Err("Error committing transaction. Row stack is empty".to_string());
            }
        }
        Ok(())
    }

    pub fn rollback_transaction(&mut self) {
        todo!()
    }

    pub fn get_column_from_row<'a>(&self, row: &'a Vec<Value>, column: &String) -> &'a Value {
        for (i, value) in row.iter().enumerate() {
            if self.get_column_names()[i] == column {
                return &value;
            }
        }
        return &Value::Null;
    }

    pub fn has_column(&self, column: &String) -> bool {
        self.get_columns().iter().any(|c| c.name == *column)
    }

    pub fn width(&self) -> usize {
        self.get_columns().len()
    }

    pub fn get_index_of_column(&self, column: &String) -> Result<usize, String> {
        for (i, c) in self.get_columns().iter().enumerate() {
            if c.name == *column {
                return Ok(i);
            }
        }
        return Err(format!(
            "Column {} does not exist in table {}",
            column, self.name
        ));
    }

    pub fn get_columns(&self) -> Vec<&ColumnDefinition> {
        self.columns.stack.last().unwrap().iter().collect()
    }

    pub fn get_columns_mut(&mut self) -> Vec<&mut ColumnDefinition> {
        self.columns.stack.last_mut().unwrap().iter_mut().collect()
    }

    pub fn get_column_names(&self) -> Vec<&String> {
        self.get_columns()
            .iter()
            .map(|column| &column.name)
            .collect()
    }

    pub fn push_column(&mut self, column: ColumnDefinition) {
        self.columns.push_column(column, false);
    }

    #[cfg(test)]
    pub fn get_columns_clone(&self) -> Vec<ColumnDefinition> {
        self.get_columns().iter().map(|c| (*c).clone()).collect()
    }
}
