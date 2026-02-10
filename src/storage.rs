use crate::value::Something;
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Row {
    values: Vec<Something>,
    listeners: Option<Vec<u32>>,
}

impl Row {
    pub fn new() -> Self {
        Row {
            values: Vec::new(),
            listeners: None,
        }
    }

    pub fn add_listener(&mut self, listener_id: u32) {
        if let Some(listeners) = &mut self.listeners {
            listeners.push(listener_id);
        } else {
            self.listeners = Some(vec![listener_id]);
        }
    }

    pub fn notify(&self, arr: &mut Vec<u32>) {
        if let Some(listeners) = &self.listeners {
            arr.extend_from_slice(listeners);
        }
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
    next_listener_id: u32,
}

impl Database {
    pub fn new() -> Self {
        Database {
            last_table_id: 0,
            tables: HashMap::default(),
            next_listener_id: 0,
        }
    }

    pub fn take_notifications(&mut self) -> Vec<u32> {
        let notifications = self
            .tables
            .values_mut()
            .flat_map(|table| {
                return table.take_notifications().into_iter();
            })
            .collect();
        return notifications;
    }

    pub fn add_listener_to(&mut self, table_id: usize, key: &Something) -> Option<u32> {
        let table = self.tables.get_mut(&table_id)?;
        let listener_id = self.next_listener_id;
        self.next_listener_id += 1;
        table.add_listener(listener_id, key)?;
        return Some(listener_id);
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

    pub fn get_table<'a>(&'a self, table_id: usize) -> Option<&'a Table> {
        let thing = self.tables.get(&table_id)?;
        return Some(thing);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    items: HashMap<Something, Row>,
    notifications: Vec<u32>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            items: HashMap::new(),
            notifications: Vec::new(),
        }
    }

    pub fn take_notifications(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.notifications)
    }

    pub fn add_listener(&mut self, listener_id: u32, key: &Something) -> Option<()> {
        let row = self.items.entry(key.clone()).or_insert_with(Row::new);
        row.add_listener(listener_id);
        return Some(());
    }

    pub fn delete_row(&mut self, key: &Something) {
        self.items.remove(key);
    }

    pub fn get(&self, key: &Something) -> Option<&Row> {
        return self.items.get(key);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Something, &Row)> {
        self.items.iter()
    }

    pub fn insert_row(&mut self, mut values: Vec<Something>) -> Option<()> {
        let key = values.pop()?;
        let row = self.items.entry(key).or_insert_with(|| {
            return Row::new();
        });
        row.values = values;
        row.notify(&mut self.notifications);
        Some(())
    }

    pub fn insert_at(&mut self, key: Something, value: Something, index: usize) {
        let e = self.items.entry(key);
        let row = e.or_insert_with(Row::new);
        row.insert_at(value, index);
        row.notify(&mut self.notifications);
    }

    pub fn remove(&mut self, key: &Something) {
        let row = self.items.remove(key);
        if let Some(row) = row {
            row.notify(&mut self.notifications);
        }
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
