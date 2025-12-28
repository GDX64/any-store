pub mod storage {
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, PartialOrd)]
    pub enum Something {
        Int(i64),
        Double(f64),
        Double2((f64, f64)),
        Text(String),
        Blob(Vec<u8>),
        Null,
    }

    impl Eq for Something {}
    impl Ord for Something {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            use Something::*;
            match (self, other) {
                (Int(a), Int(b)) => a.cmp(b),
                (Double(a), Double(b)) => {
                    a.partial_cmp(b).expect("Double values must be comparable")
                }
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

    #[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
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

    pub struct Table {
        items: BTreeMap<Something, Row>,
    }

    impl Table {
        pub fn new() -> Self {
            Table {
                items: BTreeMap::new(),
            }
        }

        pub fn insert_at(&mut self, key: Something, value: Something, index: usize) {
            let e = self.items.entry(key);
            let row = e.or_insert_with(Row::new);
            row.insert_at(value, index);
        }

        pub fn remove(&mut self, key: &Something) {
            self.items.remove(key);
        }

        pub fn get(&self, key: &Something) -> Option<&Row> {
            return self.items.get(key);
        }

        pub fn get_range(
            &self,
            start: &Something,
            end: &Something,
        ) -> impl Iterator<Item = (&Something, &Row)> {
            return self.items.range(start.clone()..end.clone());
        }

        pub fn rows_with<'a>(
            &'a self,
            f: impl Fn(&Row) -> bool + 'a,
        ) -> impl Iterator<Item = &'a Row> {
            let iter = self.items.values().filter(move |k| {
                return f(k);
            });
            return iter;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::Something;

    use super::*;

    fn setup() -> storage::Table {
        let mut store = storage::Table::new();
        let v1 = Something::Text("hello".into());
        let k1 = Something::Int(10);
        store.insert_at(k1.clone(), v1.clone(), 5);
        let v2 = Something::Text("world".into());
        let k2 = Something::Int(20);
        store.insert_at(k2.clone(), v2.clone(), 3);
        let k3 = Something::Int(30);
        store.insert_at(k3.clone(), v1.clone(), 5);

        return store;
    }

    #[test]
    fn it_works() {
        let store = setup();
        let k1 = Something::Int(10);
        let value = store.get(&k1).map(|r| r.get(5));
        let v1 = Something::Text("hello".into());
        assert_eq!(value, Some(&v1));
        let value = store.get(&Something::Int(-1));
        assert_eq!(value, None);
    }

    #[test]
    fn test_rows_with() {
        let store = setup();
        let rows = store.rows_with(|r| {
            return r.get(5) == &Something::Text("hello".into());
        });

        assert_eq!(rows.count(), 2);
    }

    #[test]
    fn test_ordering() {
        let store = setup();
        let range = store.get_range(&Something::Int(15), &Something::Int(35));
        assert!(range.count() > 1);
    }
}
