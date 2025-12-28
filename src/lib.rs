mod storage {
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, PartialOrd)]
    pub enum Something {
        Int(i64),
        Double(f64),
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
                (Text(a), Text(b)) => a.cmp(b),
                (Blob(a), Blob(b)) => a.cmp(b),
                (Int(_), _) => std::cmp::Ordering::Less,
                (Double(_), Int(_)) => std::cmp::Ordering::Greater,
                (Double(_), _) => std::cmp::Ordering::Less,
                (Text(_), Blob(_)) => std::cmp::Ordering::Less,
                (Text(_), _) => std::cmp::Ordering::Greater,
                (Blob(_), _) => std::cmp::Ordering::Greater,
            }
        }
    }

    pub struct Store {
        items: BTreeMap<Something, Vec<Option<Something>>>,
    }

    impl Store {
        pub fn new() -> Self {
            Store {
                items: BTreeMap::new(),
            }
        }

        pub fn insert_at(&mut self, key: Something, value: Something, index: usize) {
            let e = self.items.entry(key);
            let vec = e.or_insert_with(Vec::new);
            if vec.len() <= index {
                vec.resize(index + 1, None);
            }
            vec[index] = Some(value);
        }

        pub fn get_at(&self, key: &Something, index: usize) -> Option<&Something> {
            let v = self.items.get(key)?;
            let v = v.get(index)?.as_ref();
            return v;
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
}
