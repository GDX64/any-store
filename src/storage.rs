use crate::value::{ROW_TAG, Serializable, Something, TABLE_TAG};
use std::{collections::BTreeMap, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StorageOp {
    Insert {
        key: Something,
        value: Something,
        index: usize,
        version: u64,
    },
    Remove {
        key: Something,
        version: u64,
    },
}

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
    ops: Vec<StorageOp>,
    version_counter: u64,
    is_replica: bool,
}

impl Table {
    pub fn new() -> Self {
        Table {
            items: BTreeMap::new(),
            ops: Vec::new(),
            version_counter: 0,
            is_replica: false,
        }
    }

    pub fn get(&self, key: &Something) -> Option<&Row> {
        return self.items.get(key);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Something, &Row)> {
        self.items.iter()
    }

    fn next_version(&mut self) -> u64 {
        if self.is_replica {
            return 0;
        }
        self.version_counter += 1;
        return self.version_counter;
    }

    pub fn tree_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.items.hash(&mut hasher);
        return hasher.finish();
    }

    pub fn from_ops(ops: Vec<StorageOp>) -> Self {
        let mut table = Table::new();
        table.is_replica = true;
        table.apply_ops(ops);
        return table;
    }

    pub fn apply_ops(&mut self, ops: Vec<StorageOp>) {
        for op in ops {
            match op {
                StorageOp::Insert {
                    key,
                    value,
                    index,
                    version,
                } => {
                    self.update_version(version);
                    self.insert_at(key, value, index);
                }
                StorageOp::Remove { key, version } => {
                    self.update_version(version);
                    self.remove(&key);
                }
            }
        }
    }

    fn update_version(&mut self, version: u64) {
        if version == self.version_counter + 1 {
            self.version_counter = version;
        } else {
            panic!("Version mismatch in replica table");
        }
    }

    pub fn clear_ops(&mut self) {
        self.ops.clear();
    }

    pub fn take_ops(&mut self) -> Vec<StorageOp> {
        return std::mem::take(&mut self.ops);
    }

    fn push_op(&mut self, op: StorageOp) {
        if !self.is_replica {
            self.ops.push(op);
        }
    }

    pub fn insert_at(&mut self, key: Something, value: Something, index: usize) {
        let version = self.next_version();
        self.push_op(StorageOp::Insert {
            key: key.clone(),
            value: value.clone(),
            index,
            version,
        });
        let e = self.items.entry(key);
        let row = e.or_insert_with(Row::new);
        row.insert_at(value, index);
    }

    pub fn remove(&mut self, key: &Something) {
        let version = self.next_version();

        self.push_op(StorageOp::Remove {
            key: key.clone(),
            version,
        });
        self.items.remove(key);
    }

    pub fn get_range(
        &self,
        start: &Something,
        end: &Something,
    ) -> impl Iterator<Item = (&Something, &Row)> {
        return self.items.range(start.clone()..end.clone());
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
