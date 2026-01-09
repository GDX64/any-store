import type { Something } from "./WDB";

class ByteBuffer {
  arrBuffer = new ArrayBuffer(1024);
  index = 0;
  view = new DataView(this.arrBuffer);

  putU32(value: number) {
    this.view.setUint32(this.index, value, true);
    this.index += 4;
  }

  putByte(bytes: number) {
    this.view.setUint8(this.index, bytes);
    this.index += 1;
  }

  putI32(value: number) {
    this.view.setInt32(this.index, value, true);
    this.index += 4;
  }

  putF64(value: number) {
    this.view.setFloat64(this.index, value, true);
    this.index += 8;
  }
}

function serializeSomething(value: Something[]): ArrayBuffer {
  const buffer = new ByteBuffer();
  const size = value.length;
  buffer.putU32(size);
  value.forEach((item) => {
    if (item.tag === "i32") {
      buffer.putByte(0);
      buffer.putI32(item.value);
    } else if (item.tag === "f64") {
      buffer.putByte(1);
      buffer.putF64(item.value);
    } else if (item.tag === "string") {
      buffer.putByte(2);
      buffer.putU32(item.value.length);
      for (const char of item.value) {
        buffer.putByte(char.charCodeAt(0));
      }
    }
  });
  const slice = buffer.arrBuffer.slice(0, buffer.index);
  return slice;
}
