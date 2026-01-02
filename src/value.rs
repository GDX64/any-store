use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Something {
    Int(i64),
    ValueString(ValueString),
    Null,
}

impl Something {
    pub fn tag(&self) -> u8 {
        use Something::*;
        match self {
            Int(_) => 0,
            ValueString(_) => 1,
            Null => 2,
        }
    }

    pub fn string(s: String) -> Self {
        Something::ValueString(ValueString::new(&s))
    }
}

impl Hash for Something {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Something::*;
        state.write_u8(self.tag());
        match self {
            Int(v) => {
                v.hash(state);
            }
            ValueString(v) => {
                v.hash(state);
            }
            Null => {}
        }
    }
}

impl Eq for Something {}
impl Ord for Something {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use Something::*;
        match (self, other) {
            (Int(a), Int(b)) => a.cmp(b),
            // (Double(a), Double(b)) => a.partial_cmp(b).expect("Double values must be comparable"),
            // (Double2(a), Double2(b)) => {
            //     a.partial_cmp(b).expect("Double2 values must be comparable")
            // }
            // (Blob(a), Blob(b)) => a.cmp(b),
            (ValueString(a), ValueString(b)) => a.cmp(b),
            (Null, Null) => std::cmp::Ordering::Equal,
            (Null, _) => std::cmp::Ordering::Less,
            (_, Null) => std::cmp::Ordering::Greater,
            _ => panic!("Unreachable comparison case"),
        }
    }
}

impl Serializable for Something {
    fn serialize(&self, buffer: &mut ByteBuffer) {
        use Something::*;
        buffer.write_bytes(&[self.tag()]);
        match self {
            Int(v) => {
                buffer.write_bytes(&v.to_le_bytes());
            }
            ValueString(v) => {
                let bytes = v.as_bytes();
                let len = bytes.len() as u64;
                buffer.write_bytes(&len.to_le_bytes());
                buffer.write_bytes(bytes);
            }
            Null => {}
        }
    }

    fn deserialize(buffer: &mut ByteBuffer) -> Self {
        let tag = buffer.read_bytes(1)[0];
        match tag {
            0 => {
                let int_bytes = buffer.read_bytes(8);
                let int_value = i64::from_le_bytes(int_bytes.try_into().unwrap());
                Something::Int(int_value)
            }
            1 => {
                let len_bytes = buffer.read_bytes(8);
                let len = u64::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
                let str_bytes = buffer.read_bytes(len);
                let text_value = String::from_utf8(str_bytes.to_vec()).unwrap();
                Something::ValueString(ValueString::new(&text_value))
            }
            2 => Something::Null,
            _ => panic!("Unknown tag in Something deserialization"),
        }
    }
}

pub trait Serializable {
    fn serialize(&self, buffer: &mut ByteBuffer);
    fn deserialize(buffer: &mut ByteBuffer) -> Self;
}

pub struct ByteBuffer {
    buffer: Vec<u8>,
    position: usize,
}

impl ByteBuffer {
    pub fn new() -> Self {
        ByteBuffer {
            buffer: Vec::new(),
            position: 0,
        }
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn read_bytes(&mut self, length: usize) -> &[u8] {
        let start = self.position;
        let end = start + length;
        self.position = end;
        &self.buffer[start..end]
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
struct SmallString {
    data: [u8; 16],
    len: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum ValueString {
    Small(SmallString),
    Large(String),
}

impl ValueString {
    fn as_bytes(&self) -> &[u8] {
        match self {
            ValueString::Small(small) => &small.data[..small.len],
            ValueString::Large(large) => large.as_bytes(),
        }
    }
}

impl ValueString {
    fn new(s: &str) -> Self {
        if s.len() <= 16 {
            let mut data = [0u8; 16];
            data[..s.len()].copy_from_slice(s.as_bytes());
            ValueString::Small(SmallString { data, len: s.len() })
        } else {
            ValueString::Large(s.to_string())
        }
    }
}
