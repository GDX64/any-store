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
    key: Something,
}

impl Row {
    pub fn new(key: Something) -> Self {
        Row {
            values: Vec::new(),
            listeners: None,
            key,
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
}

pub enum Operation {
    Insert {
        table_id: usize,
        row_id: u32,
        value: Something,
        index: usize,
    },
    RowDelete {
        table_id: usize,
        row_id: u32,
    },
}

const NAMES_TABLE_INDEX: usize = 0;

impl Database {
    pub fn new() -> Self {
        let mut db = Database {
            last_table_id: 0,
            tables: HashMap::default(),
            next_listener_id: 0,
        };
        db.tables.insert(NAMES_TABLE_INDEX, Table::new());
        return db;
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

    pub fn operation(&mut self, op: Operation) {
        match op {
            Operation::Insert {
                table_id,
                row_id,
                value,
                index,
            } => {
                self.get_table_mut(table_id).map(|table| {
                    return table.insert_at(row_id, value, index);
                });
            }
            Operation::RowDelete { table_id, row_id } => {
                self.get_table_mut(table_id).map(|table| {
                    table.delete_row(row_id);
                });
            }
        }
    }

    pub fn remove_listener(
        &mut self,
        table_id: usize,
        row_id: u32,
        listener_id: u32,
    ) -> Option<()> {
        let listener_id = ListenerID::new(listener_id, worker_id() as u8);
        self.tables
            .get_mut(&table_id)?
            .remove_listener(row_id, listener_id);
        return Some(());
    }

    pub fn add_listener_to(&mut self, table_id: usize, row_id: u32) -> Option<ListenerID> {
        let table = self.tables.get_mut(&table_id)?;
        let listener_id = ListenerID::new(self.next_listener_id, worker_id() as u8);
        self.next_listener_id += 1;
        table.add_listener(listener_id, row_id)?;
        return Some(listener_id);
    }

    pub fn create_table(&mut self, name: Something) -> usize {
        self.last_table_id += 1;
        let table_id = self.last_table_id;
        self.tables.insert(table_id, Table::new());
        self.tables.get_mut(&NAMES_TABLE_INDEX).map(|table| {
            table.insert_at_by_key(&name, Something::Int(table_id as i32), 0);
        });
        return table_id;
    }

    pub fn get_table_id(&self, name: Something) -> Option<usize> {
        let table = self.tables.get(&NAMES_TABLE_INDEX)?;
        let row = table.get_row_by_key(&name)?;
        if let Something::Int(id) = row.get(0) {
            return Some(*id as usize);
        }
        return None;
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
    items: HashMap<Something, u32>,
    notifications: Vec<ListenerID>,
    rows: RowsCollection,
}

impl Table {
    pub fn new() -> Self {
        Table {
            items: HashMap::new(),
            notifications: Vec::new(),
            rows: RowsCollection::new(),
        }
    }

    pub fn remove_listener(&mut self, row_id: u32, listener_id: ListenerID) -> Option<()> {
        self.rows.get_mut(&row_id)?.remove_listener(listener_id);
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

    pub fn add_listener(&mut self, listener_id: ListenerID, row_id: u32) -> Option<()> {
        let row = self.rows.get_mut(&row_id)?;
        row.add_listener(listener_id);
        return Some(());
    }

    pub fn delete_row(&mut self, row_id: u32) {
        if let Some(v) = self.rows.remove(&row_id) {
            v.notify(&mut self.notifications);
            self.items.remove(&v.key);
        }
    }

    pub fn get_row(&self, row_id: u32) -> Option<&Row> {
        return self.rows.get(&row_id);
    }

    pub fn get_row_by_key(&self, key: &Something) -> Option<&Row> {
        let row_id = self.items.get(key)?;
        return self.rows.get(row_id);
    }

    pub fn create_row(&mut self, key: Something) -> u32 {
        let row = self.items.get(&key);
        if let Some(row) = row {
            return *row;
        }

        let row = Row::new(key.clone());
        let id = self.rows.insert(row);
        self.items.insert(key, id);
        return id;
    }

    pub fn insert_at(&mut self, row_id: u32, value: Something, index: usize) {
        let Some(row) = self.rows.get_mut(&row_id) else {
            return;
        };
        row.insert_at(value, index);
        row.notify(&mut self.notifications);
    }

    pub fn insert_at_by_key(&mut self, key: &Something, value: Something, index: usize) {
        let row_id = self.items.get(key);
        if let Some(row_id) = row_id {
            self.insert_at(*row_id, value, index);
        } else {
            self.create_row(key.clone());
            let row_id = self.items.get(key).unwrap();
            self.insert_at(*row_id, value, index);
        }
    }

    pub fn remove(&mut self, row_id: u32) {
        let row = self.rows.remove(&row_id);
        if let Some(row) = row {
            row.notify(&mut self.notifications);
        }
    }

    pub fn len(&self) -> usize {
        return self.items.len();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RowsCollection {
    rows: Vec<Option<Row>>,
    gaps: Vec<usize>,
}

impl RowsCollection {
    pub fn new() -> Self {
        return RowsCollection {
            rows: Vec::new(),
            gaps: Vec::new(),
        };
    }

    pub fn insert(&mut self, row: Row) -> u32 {
        if !self.gaps.is_empty() {
            let gap = self.gaps.pop().unwrap();
            self.rows[gap] = Some(row);
            return gap as u32;
        }
        self.rows.push(Some(row));
        return (self.rows.len() - 1) as u32;
    }

    pub fn get(&self, id: &u32) -> Option<&Row> {
        return self.rows.get(*id as usize)?.as_ref();
    }

    pub fn get_mut(&mut self, id: &u32) -> Option<&mut Row> {
        return self.rows.get_mut(*id as usize)?.as_mut();
    }

    pub fn remove(&mut self, id: &u32) -> Option<Row> {
        let last = self.rows.get_mut(*id as usize)?.take();
        self.gaps.push(*id as usize);
        return last;
    }
}
