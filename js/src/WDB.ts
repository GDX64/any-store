const jsHeap = new Map<number, any>();
let nextObjID = 1;

function putStringOnMap(str: string): number {
  const id = nextObjID++;
  jsHeap.set(id, str);
  return id;
}

function getStringFromMap(id: number): string | undefined {
  return jsHeap.get(id);
}

function takeObjectFromMap(id: number): any {
  const val = jsHeap.get(id);
  jsHeap.delete(id);
  return val;
}

function js_create_string(id: number): number {
  jsHeap.set(id, "");
  return id;
}

function js_next_id(): number {
  const newID = nextObjID++;
  return newID;
}

function js_put_i32(id: number, value: number): void {
  jsHeap.set(id, value);
}

function js_put_f64(id: number, value: number): void {
  jsHeap.set(id, value);
}

function js_push_to_string(stringID: number, byte: number): void {
  const str = jsHeap.get(stringID) ?? "";
  jsHeap.set(stringID, str + String.fromCharCode(byte));
}

function js_read_string_length(id: number): number {
  const str = jsHeap.get(id);
  return str ? str.length : 0;
}

function removeStringFromMap(id: number): void {
  jsHeap.delete(id);
}

function js_read_string(id: number, index: number): number {
  const str = jsHeap.get(id);
  return str ? str.charCodeAt(index) : 0;
}

export class WDB {
  private ops: Ops;
  constructor(
    private wasmInstance: WebAssembly.Instance,
    private memory: WebAssembly.Memory
  ) {
    this.ops = new Ops(wasmInstance);
  }

  static async create(data: BufferSource) {
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
        js_put_i32,
        js_put_f64,
        js_next_id,
        js_create_string,
        js_push_to_string,
        js_read_string,
        js_read_string_length,
      },
    };
    const res = await WebAssembly.instantiate(data, importObj);
    const { instance } = res;
    return new WDB(instance, memory);
  }

  createTable<T extends ColMap>(colMap: T): Table<T> {
    const id = this.ops.createTable();
    return new Table<T>(colMap, id, this);
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

  getFromTable(tableID: number, key: Something, col: number): Something | null {
    this.ops.putSomethingOnStack(key);
    this.ops.tableGetSomething(tableID, col);
    const id = this.ops.somethingPopFromStack();
    if (id > 0) {
      const value = takeObjectFromMap(id);
      const something = WDB.somethinFromValue(value);
      return something;
    }
    return null;
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

  static null(): Something {
    return { tag: "null", value: null };
  }

  static somethinFromValue(value: any): Something | null {
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

  insert(colName: keyof T, key: Something, value: Something) {
    const col = this.colMap.get(colName as string);
    this.wdb.insertOnTable(this.id, col!, key, value);
  }

  get(colName: keyof T, key: Something): Something | null {
    const col = this.colMap.get(colName as string);
    return this.wdb.getFromTable(this.id, key, col!);
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
  something_push_string(stringID: number): void;
  something_pop_string_from_stack(): number;
  table_create(): number;
  table_insert_from_stack(tableID: number, col: number): void;
  table_get_something(tableID: number, col: number): void;
  string_load(id: number): number;
  string_take(strIdx: number): number;
}

class Ops {
  constructor(private instance: WebAssembly.Instance) {}

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
    const stringID = this.createString(str);
    this.exports.something_push_string(stringID);
  }

  somethingPopFromStack(): number {
    return this.exports.something_pop_from_stack();
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
