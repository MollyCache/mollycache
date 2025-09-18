use crate::db::table::core::value::DataType;

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub constraints: Vec<ColumnConstraint>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnConstraint {
    pub constraint_type: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnStack {
    pub stack: Vec<Vec<ColumnDefinition>>,
}

impl ColumnStack {
    pub fn new(columns: Vec<ColumnDefinition>) -> Self {
        Self {
            stack: vec![columns],
        }
    }

    fn append_clone(&mut self) -> Result<(), String> {
        self.stack.push(self.peek()?.clone());
        Ok(())
    }

    pub fn push_column(&mut self, column: ColumnDefinition, is_transaction: bool) {
        if is_transaction {
            self.stack.push(self.stack.last().unwrap().clone());
        }
        self.stack.last_mut().unwrap().push(column);
    }

    pub fn rename_column(
        &mut self,
        old_column_name: &String,
        new_column_name: &String,
        is_transaction: bool,
    ) -> Result<(), String> {
        if is_transaction {
            self.append_clone()?;
        }
        let columns = self
            .peek_mut()?
            .iter_mut()
            .find(|column| column.name == *old_column_name);
        match columns {
            Some(column) => column.name = new_column_name.clone(),
            None => {
                return Err("Column does not exist".to_string());
            }
        }
        Ok(())
    }

    pub fn drop_column(
        &mut self,
        column_name: &String,
        is_transaction: bool,
    ) -> Result<(), String> {
        if is_transaction {
            self.append_clone()?;
        }
        match self.get_index_of_column(column_name) {
            Ok(index) => self.peek_mut()?.remove(index),
            Err(_) => {
                return Err("Column does not exist".to_string());
            }
        };
        Ok(())
    }

    pub fn get_index_of_column(&self, column_name: &String) -> Result<usize, String> {
        let columns = self.peek();
        match columns {
            Ok(columns) => {
                if let Some(index) = columns
                    .iter()
                    .position(|column| column.name == *column_name)
                {
                    Ok(index)
                } else {
                    Err(format!("Column `{}` does not exist", column_name))
                }
            }
            Err(_) => Err("Column stack is empty".to_string()),
        }
    }

    fn peek(&self) -> Result<&Vec<ColumnDefinition>, String> {
        self.stack.last().ok_or("Column stack is empty".to_string())
    }

    fn peek_mut(&mut self) -> Result<&mut Vec<ColumnDefinition>, String> {
        self.stack
            .last_mut()
            .ok_or("Column stack is empty".to_string())
    }
}
