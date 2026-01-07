fn read_from_value_buffer() {
    const INT_TAG: u8 = 1;
    const FLOAT_TAG: u8 = 2;
    const STRING_TAG: u8 = 3;
    struct BufferReader {
        buffer: Vec<u8>,
        position: usize,
    }
    impl BufferReader {
        fn read_u8(&mut self) -> u8 {
            if self.position >= self.buffer.len() {
                panic!("Buffer underflow")
            }
            let value = self.buffer[self.position];
            self.position += 1;
            return value;
        }

        fn read_u32(&mut self) -> u32 {
            let mut bytes = [0u8; 4];
            for i in 0..4 {
                bytes[i] = self.read_u8();
            }
            return u32::from_le_bytes(bytes);
        }
    }

    fn decode(reader: &mut BufferReader) -> Vec<Something> {
        let size = reader.read_u32() as usize;
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            let tag = reader.read_u8();
            match tag {
                INT_TAG => {
                    let int_value = reader.read_u32() as i32;
                    result.push(Something::Int(int_value));
                }
                FLOAT_TAG => {
                    let mut float_bytes = [0u8; 8];
                    for i in 0..8 {
                        float_bytes[i] = reader.read_u8();
                    }
                    let float_value = f64::from_le_bytes(float_bytes);
                    result.push(Something::Float(float_value));
                }
                STRING_TAG => {
                    let str_len = reader.read_u32() as usize;
                    let mut str_bytes = Vec::with_capacity(str_len);
                    for _ in 0..str_len {
                        str_bytes.push(reader.read_u8());
                    }
                    result.push(Something::String(str_bytes));
                }
                _ => {
                    panic!("Unknown tag");
                }
            }
        }
        return result;
    }

    VALUE_BUFFER.with(|buffer| {
        let buf = buffer.take();
        let mut reader = BufferReader {
            buffer: buf,
            position: 0,
        };
        let decoded_values = decode(&mut reader);
        SOMETHING_STACK.replace(decoded_values);
    });
}
