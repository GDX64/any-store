use crate::value::Something;
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Row {
    values: Vec<Something>,
}

impl Row {
    pub fn new() -> Self {
        Row { values: Vec::new() }
    }

    pub fn insert_at(&mut self, value: Something, index: usize) {
        if self.values.len() <= index {
            self.values.resize(index + 1, Something::Null);
        }
        self.values[index] = value;
    }

    pub fn get(&self, index: usize) -> &Something {
        return self.values.get(index).unwrap_or(&Something::Null);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Something> {
        return self.values.iter();
    }
}

pub struct Database {
    last_table_id: usize,
    tables: HashMap<usize, Table>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            last_table_id: 0,
            tables: HashMap::new(),
        }
    }

    pub fn create_table(&mut self) -> usize {
        self.last_table_id += 1;
        let table_id = self.last_table_id;
        self.tables.insert(table_id, Table::new());
        return table_id;
    }

    pub fn get_table_mut(&mut self, table_id: usize) -> Option<&mut Table> {
        let thing = self.tables.get_mut(&table_id)?;
        return Some(thing);
    }

    pub fn get_table<'a>(&'a self, table_id: usize) -> Option<&Table> {
        let thing = self.tables.get(&table_id)?;
        return Some(thing);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    items: HashMap<Something, Row>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            items: HashMap::new(),
        }
    }

    pub fn get(&self, key: &Something) -> Option<&Row> {
        return self.items.get(key);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Something, &Row)> {
        self.items.iter()
    }

    pub fn insert_row(&mut self, mut values: Vec<Something>) -> Option<()> {
        let key = values.pop()?;
        let row = Row { values };
        self.items.insert(key, row);
        Some(())
    }

    pub fn insert_at(&mut self, key: Something, value: Something, index: usize) {
        let e = self.items.entry(key);
        let row = e.or_insert_with(Row::new);
        row.insert_at(value, index);
    }

    pub fn remove(&mut self, key: &Something) {
        self.items.remove(key);
    }

    pub fn rows_with<'a>(&'a self, f: impl Fn(&Row) -> bool + 'a) -> impl Iterator<Item = &'a Row> {
        let iter = self.items.values().filter(move |k| {
            return f(k);
        });
        return iter;
    }

    pub fn len(&self) -> usize {
        return self.items.len();
    }
}
