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

function js_put_i64(id: number, value: bigint): void {
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
        js_put_i64,
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

  createTable() {
    return this.ops.createTable();
  }

  insertOnTable(
    tableID: number,
    col: number,
    key: Something,
    value: Something
  ) {
    this.putSomethingOnStack(key);
    this.putSomethingOnStack(value);
    this.ops.tableInsertFromStack(tableID, col);
  }

  getFromTable(tableID: number, key: Something, col: number): Something | null {
    this.putSomethingOnStack(key);
    this.ops.tableGetSomething(tableID, col);
    const id = this.ops.somethingPopFromStack();
    if (id > 0) {
      const value = takeObjectFromMap(id);
      const something = WDB.somethinFromValue(value);
      return something;
    }
    return null;
  }

  private putSomethingOnStack(value: Something) {
    if (value.tag === "i64") {
      this.ops.somethingPushI64ToStack(value.value);
    } else if (value.tag === "string") {
      this.ops.pushStringToStack(value.value);
    }
  }

  static i64(value: bigint): Something {
    return { tag: "i64", value };
  }

  static string(value: string): Something {
    return { tag: "string", value };
  }

  static somethinFromValue(value: any): Something | null {
    if (typeof value === "bigint") {
      return WDB.i64(value);
    } else if (typeof value === "string") {
      return WDB.string(value);
    }
    return null;
  }
}

export type Something =
  | {
      tag: "i64";
      value: bigint;
    }
  | {
      tag: "string";
      value: string;
    };

interface ExportsInterface {
  something_push_i64_to_stack(value: bigint): void;
  something_pop_i64_from_stack(): bigint;
  something_pop_from_stack(): number;
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
