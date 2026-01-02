use crate::value::Something;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    items: BTreeMap<Something, Row>,
    ops: Vec<StorageOp>,
    version_counter: u64,
    is_replica: bool,
}

impl Table {
    pub fn new(is_replica: bool) -> Self {
        Table {
            items: BTreeMap::new(),
            ops: Vec::new(),
            version_counter: 0,
            is_replica,
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
        let mut table = Table::new(true);
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
