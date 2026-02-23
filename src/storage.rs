use crate::{extern_functions::worker_id, value::Something};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, PartialOrd, Ord)]
pub struct ListenerID {
    id: u32,
    worker: u8,
}

impl ListenerID {
    fn new(id: u32, worker: u8) -> Self {
        ListenerID { id, worker }
    }

    fn is_from_worker(&self, worker_id: u8) -> bool {
        return self.worker == worker_id;
    }

    pub fn to_i32(&self) -> i32 {
        return self.id as i32;
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Row {
    values: Vec<Something>,
    listeners: Option<Vec<ListenerID>>,
}

impl Row {
    pub fn new() -> Self {
        Row {
            values: Vec::new(),
            listeners: None,
        }
    }

    pub fn remove_listener(&mut self, listener_id: ListenerID) -> Option<()> {
        if let Some(listeners) = &mut self.listeners {
            listeners.retain(|id| *id != listener_id);
            return Some(());
        }
        return None;
    }

    pub fn add_listener(&mut self, listener_id: ListenerID) {
        if let Some(listeners) = &mut self.listeners {
            listeners.push(listener_id);
        } else {
            self.listeners = Some(vec![listener_id]);
        }
    }

    pub fn notify(&self, arr: &mut Vec<ListenerID>) {
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
    pub something_stack: [Vec<Something>; 16],
    pub operation_stack: [Vec<Operation>; 16],
}

pub enum Operation {
    InsertRow {
        table_id: usize,
        data: Vec<Something>,
    },
    Insert {
        table_id: usize,
        key: Something,
        value: Something,
        index: usize,
    },
    RowDelete {
        table_id: usize,
        key: Something,
    },
}

impl Database {
    pub fn new() -> Self {
        Database {
            last_table_id: 0,
            tables: HashMap::default(),
            next_listener_id: 0,
            something_stack: Default::default(),
            operation_stack: Default::default(),
        }
    }

    pub fn take_notifications(&mut self, worker_id: u8) -> Vec<i32> {
        let notifications: HashSet<i32> = self
            .tables
            .values_mut()
            .flat_map(|table| {
                return table.take_notifications(worker_id).into_iter();
            })
            .map(|id| id.to_i32())
            .collect();

        return notifications.into_iter().collect();
    }

    pub fn remove_listener(
        &mut self,
        table_id: usize,
        key: &Something,
        listener_id: u32,
    ) -> Option<()> {
        let listener_id = ListenerID::new(listener_id, worker_id() as u8);
        self.tables
            .get_mut(&table_id)?
            .remove_listener(key, listener_id);
        return Some(());
    }

    pub fn add_listener_to(&mut self, table_id: usize, key: &Something) -> Option<ListenerID> {
        let table = self.tables.get_mut(&table_id)?;
        let listener_id = ListenerID::new(self.next_listener_id, worker_id() as u8);
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
    notifications: Vec<ListenerID>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            items: HashMap::new(),
            notifications: Vec::new(),
        }
    }

    pub fn remove_listener(&mut self, key: &Something, listener_id: ListenerID) -> Option<()> {
        self.items.get_mut(key)?.remove_listener(listener_id);
        return Some(());
    }

    fn take_notifications(&mut self, worker_id: u8) -> Vec<ListenerID> {
        let mut v = Vec::new();
        self.notifications.retain(|l| {
            if l.is_from_worker(worker_id) {
                v.push(*l);
                return false;
            }
            return true;
        });
        return v;
    }

    pub fn add_listener(&mut self, listener_id: ListenerID, key: &Something) -> Option<()> {
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
