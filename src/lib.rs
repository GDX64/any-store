mod storage {
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, PartialOrd)]
    pub enum Something {
        Int(i64),
        Double(f64),
        Double2((f64, f64)),
        Text(String),
        Blob(Vec<u8>),
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
                _ => panic!("Unreachable comparison case"),
            }
        }
    }

    pub struct Row {
        values: Vec<Option<Something>>,
    }

    impl Row {
        pub fn new() -> Self {
            Row { values: Vec::new() }
        }

        pub fn insert_at(&mut self, value: Something, index: usize) {
            if self.values.len() <= index {
                self.values.resize(index + 1, None);
            }
            self.values[index] = Some(value);
        }

        pub fn get(&self, index: usize) -> Option<&Something> {
            let v = self.values.get(index)?.as_ref();
            return v;
        }
    }

    pub struct Store {
        items: BTreeMap<Something, Row>,
    }

    impl Store {
        pub fn new() -> Self {
            Store {
                items: BTreeMap::new(),
            }
        }

        pub fn insert_at(&mut self, key: Something, value: Something, index: usize) {
            let e = self.items.entry(key);
            let row = e.or_insert_with(Row::new);
            row.insert_at(value, index);
        }

        pub fn get_at(&self, key: &Something, index: usize) -> Option<&Something> {
            let v = self.items.get(key)?;
            return v.get(index);
        }

        pub fn rows_with(&self, f: impl Fn(&Row) -> Option<()>) -> Vec<&Row> {
            return self
                .items
                .values()
                .filter_map(|k| {
                    f(k)?;
                    return Some(k);
                })
                .collect();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::Something;

    use super::*;

    #[test]
    fn it_works() {
        let mut store = storage::Store::new();

        let v1 = Something::Text("hello".into());
        let k1 = Something::Int(10);
        store.insert_at(k1.clone(), v1.clone(), 5);

        let value = store.get_at(&k1, 5);
        assert_eq!(value, Some(&v1));
        let value = store.get_at(&k1, 4);
        assert_eq!(value, None);
    }

    #[test]
    fn test_rows_with() {
        let mut store = storage::Store::new();
        let v1 = Something::Text("hello".into());
        let k1 = Something::Int(10);
        store.insert_at(k1.clone(), v1.clone(), 5);
        let v2 = Something::Text("world".into());
        let k2 = Something::Int(20);
        store.insert_at(k2.clone(), v2.clone(), 3);
        let k3 = Something::Int(30);
        store.insert_at(k3.clone(), v1.clone(), 5);

        let rows = store.rows_with(|r| {
            if r.get(5)? == &v1 {
                return Some(());
            }
            return None;
        });

        assert_eq!(rows.len(), 2);
    }
}
