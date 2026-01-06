use crate::value::{ROW_TAG, Serializable, Something, TABLE_TAG};
use std::{collections::BTreeMap, hash::Hash};

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
}

pub struct Database {
    last_table_id: usize,
    tables: BTreeMap<usize, Table>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            last_table_id: 0,
            tables: BTreeMap::new(),
        }
    }

    pub fn create_table(&mut self) -> usize {
        self.last_table_id += 1;
        let table_id = self.last_table_id;
        self.tables.insert(table_id, Table::new());
        return table_id;
    }

    pub fn get_table_mut(&mut self, table_id: usize) -> Option<&mut Table> {
        return self.tables.get_mut(&table_id);
    }

    pub fn get_table(&self, table_id: usize) -> Option<&Table> {
        return self.tables.get(&table_id);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    items: BTreeMap<Something, Row>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            items: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &Something) -> Option<&Row> {
        return self.items.get(key);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Something, &Row)> {
        self.items.iter()
    }

    pub fn tree_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.items.hash(&mut hasher);
        return hasher.finish();
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

impl Serializable for Row {
    fn deserialize(buffer: &mut crate::value::ByteBuffer) -> Self {
        let len_bytes = buffer.read_bytes(8);
        let len = u64::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
        let mut row = Row::new();
        for _ in 0..len {
            let value = Something::deserialize(buffer);
            row.values.push(value);
        }
        return row;
    }

    fn serialize(&self, buffer: &mut crate::value::ByteBuffer) {
        let len = self.values.len() as u64;
        buffer.write_bytes(&[ROW_TAG]);
        buffer.write_bytes(&len.to_le_bytes());
        for v in &self.values {
            v.serialize(buffer);
        }
    }
}

impl Serializable for Table {
    fn deserialize(buffer: &mut crate::value::ByteBuffer) -> Self {
        let is_table = buffer.read_bytes(1)[0] == TABLE_TAG;
        if !is_table {
            panic!("Expected TABLE_TAG in Table deserialization");
        }
        let mut table = Table::new();
        let num_items_bytes = buffer.read_bytes(8);
        let num_items = u64::from_le_bytes(num_items_bytes.try_into().unwrap()) as usize;
        for _ in 0..num_items {
            let key = Something::deserialize(buffer);
            let row = Row::deserialize(buffer);
            table.items.insert(key, row);
        }
        return table;
    }

    fn serialize(&self, buffer: &mut crate::value::ByteBuffer) {
        buffer.write_bytes(&[TABLE_TAG]);
        let num_items = self.items.len() as u64;
        buffer.write_bytes(&num_items.to_le_bytes());
        for (k, v) in &self.items {
            k.serialize(buffer);
            v.serialize(buffer);
        }
    }
}
