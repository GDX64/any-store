const jsStack: any[] = [];

function pushToStringStack(str: string) {
  jsStack.push(str);
}

function popObjectFromStack(): any {
  const val = jsStack.pop();
  return val;
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

function safe_js_pop_stack(): void {
  jsStack.pop();
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
};

export class WDB {
  private ops: Ops;
  constructor(
    private wasmInstance: WebAssembly.Instance,
    private memory: WebAssembly.Memory,
    private module: WebAssembly.Module
  ) {
    this.ops = new Ops(wasmInstance);
  }

  static async create(data: BufferSource) {
    const memory = new WebAssembly.Memory({
      initial: 20,
      maximum: 10_000,
      shared: true,
    });
    const importObj = {
      env: {
        memory,
      },
      ops,
    };
    const res = await WebAssembly.instantiate(data, importObj);
    const { instance, module } = res;
    return new WDB(instance, memory, module);
  }

  static async fromModule(
    module: WebAssembly.Module,
    memory: WebAssembly.Memory
  ) {
    const instance = await WebAssembly.instantiate(module, {
      env: {
        memory,
      },
      ops,
    });
    return new WDB(instance, memory, module);
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
    value: Something
  ) {
    this.ops.putSomethingOnStack(key);
    this.ops.putSomethingOnStack(value);
    this.ops.tableInsertFromStack(tableID, col);
  }

  getFromTable(
    tableID: number,
    key: Something,
    col: number
  ): Something["value"] | null {
    this.ops.putSomethingOnStack(key);
    this.ops.tableGetSomething(tableID, col);
    this.ops.somethingPopFromStack();
    const value = popObjectFromStack();
    return value ?? null;
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
    }
    return null;
  }
}

type ColMap = Record<string, Something["tag"]>;

class Table<T extends ColMap> {
  colMap: Map<string, number> = new Map();
  constructor(colMap: T, private id: number, private wdb: WDB) {
    Object.keys(colMap).forEach((colName, index) => {
      this.colMap.set(colName, index);
    });
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

  row(key: Something) {
    return new Row<T>(this, key);
  }
}

class Row<T extends ColMap> {
  constructor(private table: Table<T>, private key: Something) {}

  get<K extends keyof T>(colName: K): Something["value"] | null {
    return this.table.get(this.key, colName);
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
    };

interface ExportsInterface {
  something_push_i32_to_stack(value: number): void;
  something_pop_i32_from_stack(): number;
  something_push_f64_to_stack(value: number): void;
  something_pop_f64_from_stack(): number;
  something_pop_from_stack(): number;
  something_push_null_to_stack(): void;
  something_push_string(): void;
  table_create(): number;
  table_insert_from_stack(tableID: number, col: number): void;
  table_get_something(tableID: number, col: number): void;
  table_insert_row(tableID: number): void;
  string_take(strIdx: number): number;
  start(): void;
}

class Ops {
  constructor(private instance: WebAssembly.Instance) {
    this.exports.start();
  }

  get exports(): ExportsInterface {
    return this.instance.exports as unknown as ExportsInterface;
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
    } else if (value.tag === "null") {
      this.pushNullToStack();
    }
  }

  pushNullToStack(): void {
    this.exports.something_push_null_to_stack();
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

  tableInsertFromStack(tableID: number, col: number): void {
    this.exports.table_insert_from_stack(tableID, col);
  }

  tableGetSomething(tableID: number, col: number): void {
    this.exports.table_get_something(tableID, col);
  }
}
