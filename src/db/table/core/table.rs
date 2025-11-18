use crate::db::table::core::column::ColumnDefinition;
use crate::db::table::core::column::ColumnStack;
use crate::db::table::core::row::Row;
use crate::db::table::core::row::RowStack;
use crate::db::table::core::value::Value;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Table {
    pub name: NameStack,
    pub columns: ColumnStack,
    pub rows: Vec<RowStack>,
    length: usize,
}

#[derive(Debug)]
pub struct NameStack {
    pub stack: Vec<String>,
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
            name: NameStack { stack: vec![name] },
            columns: ColumnStack::new(columns),
            rows: vec![],
            length: 0,
        }
    }

    pub fn name(&self) -> Result<&String, String> {
        self.name
            .stack
            .last()
            .ok_or_else(|| "Error fetching table name.".to_string())
    }

    pub fn change_name(&mut self, new_name: String, is_transaction: bool) {
        if is_transaction {
            self.name.stack.push(new_name);
        } else {
            self.name.stack = vec![new_name];
        }
    }

    pub fn get(&self, i: usize) -> Option<&Row> {
        if i < self.length {
            self.rows.get(i)?.stack.last()
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Row> {
        self.rows
            .iter()
            .take(self.length)
            .map(|s| s.stack.last().unwrap())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Row> {
        self.rows
            .iter_mut()
            .take(self.length)
            .map(|s| s.stack.last_mut().unwrap())
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = length;
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.rows.swap(a, b);
    }

    #[cfg(test)]
    pub fn get_rows_clone(&self) -> Vec<Row> {
        self.rows
            .iter()
            .take(self.length)
            .map(|s| s.stack.last().unwrap().clone())
            .collect()
    }

    pub fn get_rows(&self) -> Vec<&Row> {
        self.rows
            .iter()
            .take(self.length)
            .map(|s| s.stack.last().unwrap())
            .collect()
    }

    pub fn get_rows_mut(&mut self) -> Vec<&mut Row> {
        self.rows
            .iter_mut()
            .take(self.length)
            .map(|s| s.stack.last_mut().unwrap())
            .collect()
    }

    pub fn get_row_stacks_mut(&mut self) -> &mut Vec<RowStack> {
        &mut self.rows
    }

    #[cfg(test)]
    pub fn get_row_stacks_clone(&self) -> Vec<RowStack> {
        self.rows.clone()
    }

    pub fn set_rows(&mut self, rows: Vec<Row>) {
        self.length = rows.len();
        self.rows = rows.into_iter().map(|r| RowStack::new(r)).collect();
    }

    pub fn push(&mut self, row: Row) {
        self.length += 1;
        self.rows.push(RowStack::new(row));
    }

    pub fn pop(&mut self) -> Option<Row> {
        if self.length == 0 {
            return None;
        }
        self.length -= 1;
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
        if self.columns.stack.len() > 1 {
            let last_column_stack = self.columns.stack.pop().unwrap();
            self.columns = ColumnStack::new(last_column_stack);
        }
        if self.name.stack.len() > 1 {
            let last_name = self.name.stack.pop().unwrap();
            self.name = NameStack {
                stack: vec![last_name],
            };
        }
        Ok(())
    }

    pub fn rollback_columns(&mut self) {
        self.columns.stack.pop();
    }

    pub fn rollback_all_rows(&mut self) {
        for row_stack in self.rows.iter_mut() {
            row_stack.stack.pop();
        }
    }

    pub fn rollback_name(&mut self) {
        self.name.stack.pop();
    }

    pub fn get_column_from_row<'a>(
        &self,
        row: &'a Vec<Value>,
        column: &String,
    ) -> Result<&'a Value, String> {
        for (i, value) in row.iter().enumerate() {
            if self.get_column_names()?[i] == column {
                return Ok(&value);
            }
        }
        return Ok(&Value::Null);
    }

    pub fn has_column(&self, column: &String) -> Result<bool, String> {
        Ok(self.get_columns()?.iter().any(|c| c.name == *column))
    }

    pub fn width(&self) -> Result<usize, String> {
        Ok(self.get_columns()?.len())
    }

    pub fn get_index_of_column(&self, column: &String) -> Result<usize, String> {
        for (i, c) in self.get_columns()?.iter().enumerate() {
            if c.name == *column {
                return Ok(i);
            }
        }
        return Err(format!(
            "Column {} does not exist in table {}",
            column,
            self.name()?
        ));
    }

    pub fn get_columns(&self) -> Result<Vec<&ColumnDefinition>, String> {
        Ok(self
            .columns
            .stack
            .last()
            .ok_or_else(|| "Column stack is empty".to_string())?
            .iter()
            .collect())
    }

    pub fn get_columns_mut(&mut self) -> Result<Vec<&mut ColumnDefinition>, String> {
        Ok(self
            .columns
            .stack
            .last_mut()
            .ok_or_else(|| "Column stack is empty".to_string())?
            .iter_mut()
            .collect())
    }

    pub fn get_column_names(&self) -> Result<Vec<&String>, String> {
        Ok(self
            .get_columns()?
            .iter()
            .map(|column| &column.name)
            .collect())
    }

    pub fn push_column(&mut self, column: ColumnDefinition, is_transaction: bool) {
        self.columns.push_column(column, is_transaction);
    }

    #[cfg(test)]
    pub fn get_columns_clone(&self) -> Result<Vec<ColumnDefinition>, String> {
        Ok(self.get_columns()?.iter().map(|c| (*c).clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::value::DataType;

    #[test]
    fn pop_on_empty_table_does_not_underflow() {
        let columns = vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: DataType::Integer,
            constraints: vec![],
        }];
        let mut table = Table::new("test".to_string(), columns);
        let row = Row(vec![Value::Integer(42)]);
        table.push(row.clone());
        assert_eq!(table.len(), 1);

        let popped = table.pop();
        assert_eq!(popped, Some(row));
        assert_eq!(table.len(), 0);

        let result = table.pop();
        assert_eq!(result, None);
        assert_eq!(table.len(), 0);
    }
}
