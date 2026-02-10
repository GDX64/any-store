import wasmModule from "../../target/wasm32-unknown-unknown/release/any_store.wasm?url";

const jsStack: any[] = [];

function pushToStringStack(str: string) {
  jsStack.push(str);
}

function pushBlobToStack(blob: Uint8Array) {
  jsStack.push(blob);
}

function getWholeStack(): any[] {
  return jsStack.splice(0, jsStack.length);
}

function popObjectFromStack(): any {
  const val = jsStack.pop();
  if (typeof val === "object") {
    return val.value;
  }
  return val;
}

function js_push_null(): void {
  jsStack.push(null);
}

function js_put_i32(value: number): void {
  jsStack.push(value);
}

function js_put_f64(value: number): void {
  jsStack.push(value);
}

function js_push_string_to_stack() {
  jsStack.push("");
}

function js_log_stack_value(): void {
  const val = jsStack.pop();
  console.log("WASM LOG:", val);
}

function js_push_to_string(byte: number): void {
  jsStack[jsStack.length - 1] += String.fromCharCode(byte);
}

function js_pop_stack(): void {
  jsStack.pop();
}

function js_read_string_length(): number {
  return jsStack.at(-1)?.length ?? 0;
}

function js_read_string(index: number): number {
  return jsStack.at(-1)?.charCodeAt(index) ?? 0;
}

function js_read_blob_length(): number {
  return jsStack.at(-1)?.length ?? 0;
}

function js_read_blob_byte(index: number): number {
  return jsStack.at(-1)?.[index] ?? 0;
}

function js_performance_now() {
  return performance.now();
}

function js_create_blob(size: number) {
  jsStack.push({ value: new Uint8Array(size), index: 0 });
}

function js_push_to_blob(byte: number) {
  const blob = jsStack.at(-1) as { value: Uint8Array; index: number };
  blob.value[blob.index] = byte;
  blob.index += 1;
}

const ops = {
  js_put_i32,
  js_put_f64,
  js_push_to_string,
  js_read_string_length,
  js_read_string,
  js_pop_stack,
  js_push_string_to_stack,
  js_log_stack_value,
  js_push_null,
  js_performance_now,
  js_create_blob,
  js_push_to_blob,
  js_read_blob_length,
  js_read_blob_byte,
};

export class WDB {
  private ops: Ops;
  constructor(
    wasmInstance: WebAssembly.Instance,
    private memory: WebAssembly.Memory,
    private module: WebAssembly.Module,
  ) {
    this.ops = new Ops(wasmInstance);
  }

  static async create(id: number = 0, bufferSource?: BufferSource) {
    let data: BufferSource;
    if (bufferSource) {
      data = bufferSource;
    } else {
      const response = await fetch(wasmModule);
      data = await response.arrayBuffer();
    }
    const memory = new WebAssembly.Memory({
      initial: 20,
      maximum: 10_000,
      shared: true,
    });
    const worker_id = () => id;
    const importObj = {
      env: {
        memory,
        worker_id,
      },
      ops,
    };
    const res = await WebAssembly.instantiate(data, importObj);
    const { instance, module } = res;
    return new WDB(instance, memory, module);
  }

  static async fromModule(
    module: WebAssembly.Module,
    memory: WebAssembly.Memory,
    id: number,
  ) {
    const worker_id = () => id;
    const instance = await WebAssembly.instantiate(module, {
      env: {
        memory,
        worker_id,
      },
      ops,
    });
    return new WDB(instance, memory, module);
  }

  commit() {
    return this.ops.commit();
  }

  getTable<T extends ColMap>(tableID: number, colMap: T): Table<T> {
    return new Table<T>(colMap, tableID, this);
  }

  memSize() {
    return this.memory.buffer.byteLength;
  }

  createTable<T extends ColMap>(colMap: T): Table<T> {
    const id = this.ops.createTable();
    return new Table<T>(colMap, id, this);
  }

  insertRowOnTable(tableID: number, key: Something, values: Something[]) {
    values.forEach((value) => {
      this.ops.putSomethingOnStack(value);
    });
    this.ops.putSomethingOnStack(key);
    this.ops.exports.table_insert_row(tableID);
  }

  insertOnTable(
    tableID: number,
    col: number,
    key: Something,
    value: Something,
  ) {
    this.ops.putSomethingOnStack(key);
    this.ops.putSomethingOnStack(value);
    this.ops.tableInsert(tableID, col);
  }

  getFromTable(
    tableID: number,
    key: Something,
    col: number,
  ): Something["value"] | null {
    this.ops.putSomethingOnStack(key);
    this.ops.tableGetSomething(tableID, col);
    const value = popObjectFromStack();
    return value ?? null;
  }

  deleteRowFromTable(tableID: number, key: Something) {
    this.ops.putSomethingOnStack(key);
    this.ops.deleteRowFromTable(tableID);
  }

  getRowFromTable(tableID: number, key: Something): Something["value"][] {
    this.ops.putSomethingOnStack(key);
    this.ops.getRowFromTable(tableID);
    return getWholeStack();
  }

  getMemory(): WebAssembly.Memory {
    return this.memory;
  }

  getModule(): WebAssembly.Module {
    return this.module;
  }

  static i32(value: number): Something {
    return { tag: "i32", value };
  }

  static f64(value: number): Something {
    return { tag: "f64", value };
  }

  static blob(value: Uint8Array): Something {
    return { tag: "blob", value };
  }

  static string(value: string): Something {
    return { tag: "string", value };
  }

  static stack() {
    return jsStack;
  }

  static null(): Something {
    return { tag: "null", value: null };
  }

  static somethingFromValue(value: any): Something | null {
    if (typeof value === "number") {
      return WDB.f64(value);
    } else if (typeof value === "string") {
      return WDB.string(value);
    } else if (value === null) {
      return { tag: "null", value: null };
    } else if (value instanceof Uint8Array) {
      return WDB.blob(value);
    }
    return null;
  }
}

type ColMap = Record<string, Something["tag"]>;

export class Table<T extends ColMap> {
  colMap: Map<string, number> = new Map();
  constructor(
    colMap: T,
    private id: number,
    private wdb: WDB,
  ) {
    Object.keys(colMap).forEach((colName, index) => {
      this.colMap.set(colName, index);
    });
  }

  getRow(key: Something): Something["value"][] {
    return this.wdb.getRowFromTable(this.id, key);
  }

  insert(key: Something, value: Something, colName: keyof T) {
    const col = this.colMap.get(colName as string);
    this.wdb.insertOnTable(this.id, col!, key, value);
  }

  insertRow(key: Something, values: Something[]) {
    this.wdb.insertRowOnTable(this.id, key, values);
  }

  get(key: Something, colName: keyof T): Something["value"] | null {
    const col = this.colMap.get(colName as string);
    return this.wdb.getFromTable(this.id, key, col!);
  }

  deleteRow(key: Something) {
    this.wdb.deleteRowFromTable(this.id, key);
  }

  row(key: Something) {
    return new Row<T>(this, key);
  }
}

export class Row<T extends ColMap> {
  constructor(
    private table: Table<T>,
    private key: Something,
  ) {}

  get<K extends keyof T>(colName: K): Something["value"] | null {
    return this.table.get(this.key, colName);
  }

  delete() {
    return this.table.deleteRow(this.key);
  }

  update<K extends keyof T>(colName: K, value: Something) {
    this.table.insert(this.key, value, colName);
  }

  getRow(): Something["value"][] {
    return this.table.getRow(this.key);
  }
}

export type Something =
  | {
      tag: "i32";
      value: number;
    }
  | {
      tag: "string";
      value: string;
    }
  | {
      tag: "null";
      value: null;
    }
  | {
      tag: "f64";
      value: number;
    }
  | {
      tag: "blob";
      value: Uint8Array;
    };

interface ExportsInterface {
  something_push_i32_to_stack(value: number): void;
  something_pop_i32_from_stack(): number;
  something_push_f64_to_stack(value: number): void;
  something_pop_f64_from_stack(): number;
  something_pop_from_stack(): number;
  something_push_null_to_stack(): void;
  something_push_string(): void;
  something_push_blob(size: number): void;
  table_create(): number;
  table_insert(tableID: number, col: number): void;
  table_get_row(tableID: number): void;
  commit_ops(): void;
  table_get_something(tableID: number, col: number): void;
  table_insert_row(tableID: number): void;
  string_take(strIdx: number): number;
  delete_row_from_table(tableID: number): void;
  start(): void;
}

class Ops {
  exports: ExportsInterface;
  constructor(instance: WebAssembly.Instance) {
    this.exports = instance.exports as unknown as ExportsInterface;
    this.exports.start();
  }

  commit() {
    return this.exports.commit_ops();
  }

  createTable() {
    return this.exports.table_create();
  }

  putSomethingOnStack(value: Something) {
    if (value.tag === "i32") {
      this.somethingPushi32ToStack(value.value);
    } else if (value.tag === "string") {
      this.pushStringToStack(value.value);
    } else if (value.tag === "f64") {
      this.somethingPushf64ToStack(value.value);
    } else if (value.tag === "blob") {
      this.somethingPushBlobToStack(value.value);
    } else if (value.tag === "null") {
      this.pushNullToStack();
    }
  }

  deleteRowFromTable(tableID: number): void {
    this.exports.delete_row_from_table(tableID);
  }

  getRowFromTable(tableID: number): void {
    this.exports.table_get_row(tableID);
  }

  pushNullToStack(): void {
    this.exports.something_push_null_to_stack();
  }

  somethingPushBlobToStack(value: Uint8Array): void {
    pushBlobToStack(value);
    this.exports.something_push_blob(value.length);
  }

  somethingPushf64ToStack(value: number): void {
    this.exports.something_push_f64_to_stack(value);
  }

  somethingPushi32ToStack(value: number): void {
    this.exports.something_push_i32_to_stack(value);
  }

  somethingPopi32FromStack(): number {
    return this.exports.something_pop_i32_from_stack();
  }

  pushStringToStack(str: string): void {
    pushToStringStack(str);
    this.exports.something_push_string();
  }

  somethingPopFromStack() {
    return this.exports.something_pop_from_stack();
  }

  tableInsert(tableID: number, col: number): void {
    this.exports.table_insert(tableID, col);
  }

  tableGetSomething(tableID: number, col: number): void {
    this.exports.table_get_something(tableID, col);
  }
}
