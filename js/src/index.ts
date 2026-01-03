import fs from "fs";

const stringMap = new Map<number, string>();
let nextStringID = 1;

function putStringOnMap(str: string): number {
  const id = nextStringID++;
  stringMap.set(id, str);
  return id;
}

function getStringFromMap(id: number): string | undefined {
  return stringMap.get(id);
}

function js_create_string(): number {
  const id = nextStringID++;
  stringMap.set(id, "");
  return id;
}

function js_push_to_string(stringID: number, byte: number): void {
  const str = stringMap.get(stringID) || "";
  stringMap.set(stringID, str + String.fromCharCode(byte));
}

function js_read_string_length(id: number): number {
  const str = stringMap.get(id);
  return str ? str.length : 0;
}

function removeStringFromMap(id: number): void {
  stringMap.delete(id);
}

function js_read_string(id: number, index: number): number {
  const str = stringMap.get(id);
  return str ? str.charCodeAt(index) : 0;
}

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
    ops: {
      js_create_string,
      js_push_to_string,
      js_read_string,
      js_read_string_length,
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

  //key
  wdb.somethingPushI64ToStack(123n);
  //value
  wdb.pushStringToStack("hello world");
  wdb.tableInsertFromStack(tableID, 1);
  //key again
  wdb.somethingPushI64ToStack(123n);
  wdb.tableGetSomething(tableID, 1);
  const resultID = wdb.popStringFromStack();
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

  pushStringToStack(str: string): void {
    const stringID = this.createString(str);
    this.exports.something_push_string(stringID);
  }

  popStringFromStack(): number {
    return this.exports.something_pop_string_from_stack();
  }

  tableInsertFromStack(tableID: number, col: number): void {
    this.exports.table_insert_from_stack(tableID, col);
  }

  tableGetSomething(tableID: number, col: number): void {
    this.exports.table_get_something(tableID, col);
  }

  loadString(id: number): number {
    return this.exports.string_load(id);
  }

  createString(str: string): number {
    const id = putStringOnMap(str);
    const strID = this.loadString(id);
    removeStringFromMap(id);
    return strID;
  }

  getString(stringID: number): string {
    const jsID = this.exports.string_take(stringID);
    const str = getStringFromMap(jsID);
    removeStringFromMap(jsID);
    return str || "";
  }
}

interface ExportsInterface {
  something_push_i64_to_stack(value: bigint): void;
  something_pop_i64_from_stack(): bigint;
  something_push_string(stringID: number): void;
  something_pop_string_from_stack(): number;
  table_create(): number;
  table_insert_from_stack(tableID: number, col: number): void;
  table_get_something(tableID: number, col: number): void;
  string_load(id: number): number;
  string_take(strIdx: number): number;
}
