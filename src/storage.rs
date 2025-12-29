use std::{collections::BTreeMap, hash::Hash};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Something {
    Int(i64),
    Double(f64),
    Double2((f64, f64)),
    Text(String),
    Blob(Vec<u8>),
    Null,
}

impl Hash for Something {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Something::*;
        match self {
            Int(v) => {
                state.write_u8(0);
                v.hash(state);
            }
            Double(v) => {
                state.write_u8(1);
                state.write(&v.to_le_bytes());
            }
            Double2((v1, v2)) => {
                state.write_u8(2);
                state.write(&v1.to_le_bytes());
                state.write(&v2.to_le_bytes());
            }
            Text(v) => {
                state.write_u8(3);
                v.hash(state);
            }
            Blob(v) => {
                state.write_u8(4);
                v.hash(state);
            }
            Null => {
                state.write_u8(5);
            }
        }
    }
}

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

impl Eq for Something {}
impl Ord for Something {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use Something::*;
        match (self, other) {
            (Int(a), Int(b)) => a.cmp(b),
            (Double(a), Double(b)) => a.partial_cmp(b).expect("Double values must be comparable"),
            (Double2(a), Double2(b)) => {
                a.partial_cmp(b).expect("Double2 values must be comparable")
            }
            (Text(a), Text(b)) => a.cmp(b),
            (Blob(a), Blob(b)) => a.cmp(b),
            (Null, Null) => std::cmp::Ordering::Equal,
            (Null, _) => std::cmp::Ordering::Less,
            (_, Null) => std::cmp::Ordering::Greater,
            _ => panic!("Unreachable comparison case"),
        }
    }
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

pub trait DbTable {
    fn get(&self, key: &Something) -> Option<&Row>;
    fn iter(&self) -> impl Iterator<Item = (&Something, &Row)>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    items: BTreeMap<Something, Row>,
    ops: Vec<StorageOp>,
    version_counter: u64,
    is_replica: bool,
}

impl DbTable for Table {
    fn get(&self, key: &Something) -> Option<&Row> {
        return self.items.get(key);
    }

    fn iter(&self) -> impl Iterator<Item = (&Something, &Row)> {
        self.items.iter()
    }
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
