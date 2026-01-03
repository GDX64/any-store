import fs from "fs";

async function main() {
  const memory = new WebAssembly.Memory({
    initial: 20,
    maximum: 40,
    shared: true,
  });
  const importObj = {
    env: {
      memory,
    },
  };

  const data = fs.readFileSync(
    "../target/wasm32-unknown-unknown/release/any_store.wasm"
  );

  const res = await WebAssembly.instantiate(data, importObj);
  const { instance } = res;
  const wdb = new WDB(instance, memory);
  //key
  wdb.somethingPushI64ToStack(43n);
  //value
  wdb.somethingPushI64ToStack(84n);
  const tableID = wdb.createTable();
  const col = 0;
  wdb.tableInsertFromStack(tableID, col);

  //key again
  wdb.somethingPushI64ToStack(43n);
  wdb.tableGetSomething(tableID, col);
  const result = wdb.somethingPopI64FromStack();
  console.log("result:", result); // should be 84

  const stringID = wdb.createString("hello world");

  //key
  wdb.somethingPushI64ToStack(123n);
  //value
  wdb.somethingPushString(stringID);
  wdb.tableInsertFromStack(tableID, 1);
  //key again
  wdb.somethingPushI64ToStack(123n);
  wdb.tableGetSomething(tableID, 1);
  const resultID = wdb.somethingPopStringFromStack();
  const decodedString = wdb.getString(resultID);
  console.log("result string:", decodedString); // should be "hello world"
}

main();

class WDB {
  constructor(
    private wasmInstance: WebAssembly.Instance,
    private memory: WebAssembly.Memory
  ) {}

  private get exports(): ExportsInterface {
    return this.wasmInstance.exports as unknown as ExportsInterface;
  }

  createTable() {
    return this.exports.table_create();
  }

  somethingPushI64ToStack(value: bigint): void {
    this.exports.something_push_i64_to_stack(value);
  }

  somethingPopI64FromStack(): bigint {
    return this.exports.something_pop_i64_from_stack();
  }

  somethingPushString(stringID: number): void {
    this.exports.something_push_string(stringID);
  }

  somethingPopStringFromStack(): number {
    return this.exports.something_pop_string_from_stack();
  }

  stringCreate(len: number): number {
    return this.exports.string_create(len);
  }

  stringGetPointer(stringID: number): number {
    return this.exports.string_get_pointer(stringID);
  }

  tableInsertFromStack(tableID: number, col: number): void {
    this.exports.table_insert_from_stack(tableID, col);
  }

  tableGetSomething(tableID: number, col: number): void {
    this.exports.table_get_something(tableID, col);
  }

  createString(str: string): number {
    const stringID = this.stringCreate(str.length);
    const stringPtr = this.stringGetPointer(stringID);
    const arr = new Uint8Array(this.memory.buffer);
    const strBytes = new TextEncoder().encode(str);
    arr.set(strBytes, stringPtr);
    return stringID;
  }

  getString(stringID: number): string {
    const length = this.stringGetLength(stringID);
    const pointer = this.stringGetPointer(stringID);
    const bytes = new Uint8Array(this.memory.buffer, pointer, length);
    return new TextDecoder().decode(bytes);
  }

  stringGetLength(stringID: number): number {
    return this.exports.string_get_length(stringID);
  }
}

interface ExportsInterface {
  something_push_i64_to_stack(value: bigint): void;
  something_pop_i64_from_stack(): bigint;
  something_push_string(stringID: number): void;
  something_pop_string_from_stack(): number;
  string_create(len: number): number;
  string_get_pointer(stringID: number): number;
  table_create(): number;
  table_insert_from_stack(tableID: number, col: number): void;
  table_get_something(tableID: number, col: number): void;
  string_get_length(stringID: number): number;
}
