use std::hash::Hash;

const INT_TAG: u8 = 0;
const VALUE_STRING_TAG: u8 = 1;
const NULL_TAG: u8 = 2;
pub const ROW_TAG: u8 = 3;
pub const TABLE_TAG: u8 = 4;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Something {
    Int(i64),
    String(String),
    Null,
}

impl Default for Something {
    fn default() -> Self {
        Something::Null
    }
}

impl Something {
    pub fn tag(&self) -> u8 {
        use Something::*;
        match self {
            Int(_) => INT_TAG,
            String(_) => VALUE_STRING_TAG,
            Null => NULL_TAG,
        }
    }

    pub fn string(s: String) -> Self {
        Something::String(s)
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
            String(v) => {
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
            (String(a), String(b)) => a.cmp(b),
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
            String(v) => {
                let bytes = v.as_bytes();
                let len = bytes.len() as u8;
                buffer.write_bytes(&[len]);
                buffer.write_bytes(bytes);
            }
            Null => {}
        }
    }

    fn deserialize(buffer: &mut ByteBuffer) -> Self {
        let tag = buffer.read_bytes(1)[0];
        match tag {
            INT_TAG => {
                let int_bytes = buffer.read_bytes(8);
                let int_value = i64::from_le_bytes(int_bytes.try_into().unwrap());
                Something::Int(int_value)
            }
            VALUE_STRING_TAG => {
                let len_bytes = buffer.read_bytes(1);
                let len = len_bytes[0] as usize;
                let str_bytes = buffer.read_bytes(len);
                let text_value = String::from_utf8(str_bytes.to_vec()).unwrap();
                Something::String(text_value)
            }
            NULL_TAG => Something::Null,
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

    pub fn from_vec(data: Vec<u8>) -> Self {
        ByteBuffer {
            buffer: data,
            position: 0,
        }
    }

    pub fn read_u8(&mut self) -> u8 {
        let byte = self.buffer[self.position];
        self.position += 1;
        byte
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn read_i64(&mut self) -> i64 {
        let bytes = &self.buffer[self.position..self.position + 8];
        self.position += 8;
        i64::from_le_bytes(bytes.try_into().unwrap())
    }

    pub fn write_i64(&mut self, value: i64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn reset(&mut self) {
        self.position = 0;
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
