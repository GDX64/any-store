import initModule, { type InitOutput } from "../pkg/any_store";

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
  if (val && typeof val === "object") {
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

function startWorkerID(workerID: number) {
  (globalThis as any).unsafe_worker_id = () => workerID;
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
  unsafe_worker_id: () => 0,
};

for (const op in ops) {
  (globalThis as any)[op] = (ops as any)[op];
}

type WorkerData = {
  memory: WebAssembly.Memory;
  workerID: number;
};

export class AnyStore {
  private ops: Ops;
  private listeners: Map<number, () => void> = new Map();
  private workerID: number = 0;

  constructor(
    out: InitOutput,
    private memory: WebAssembly.Memory,
  ) {
    this.ops = new Ops(out);
  }

  private async lockAsync() {
    while (true) {
      const success = this.ops.exports.try_lock();
      if (success) {
        return;
      } else {
        const pointer = this.ops.exports.lock_pointer();
        const array = new Int32Array(this.memory.buffer, pointer, 1);
        await Atomics.waitAsync(array, 0, 0);
      }
    }
  }

  createRow(key: Something, tableID: number): number {
    this.ops.putSomethingOnStack(key.value, key.tag);
    return this.ops.exports.table_create_row(tableID);
  }

  /**
   * This function will probably be less performant than withLock
   * but it wont block the current thread in the case the lock
   * cant be acquired
   */
  async withLockAsync<T>(fn: () => Promise<T>): Promise<T> {
    try {
      await this.lockAsync();
      const result = await fn();
      return result;
    } finally {
      this.unlock();
    }
  }

  /**
   * This function will block the thread in
   * web workers in the case some worker has the lock
   * it wont burn CPU on workers though because of Atomic.wait
   * On the main thread it will spin loop and burn CPU until it gets the lock
   */
  withLock<T>(fn: () => T): T {
    try {
      this.lock();
      const result = fn();
      return result;
    } finally {
      this.unlock();
    }
  }

  private lock() {
    this.ops.lock();
  }

  private unlock() {
    this.ops.unlock();
  }

  static async create() {
    const memory = new WebAssembly.Memory({
      initial: 20,
      maximum: 1000,
      shared: true,
    });
    const mod = await initModule({ memory });
    return new AnyStore(mod, memory);
  }

  static async fromModule(workerData: WorkerData) {
    startWorkerID(workerData.workerID);
    const mod = await initModule({ memory: workerData.memory });
    return new AnyStore(mod, workerData.memory);
  }

  getTable<T extends ColMap>(name: string, colMap: T): Table<T> | null {
    const id = this.ops.getTableIDFromName(name);
    if (!id) {
      return null;
    }
    return new Table<T>(colMap, id, this);
  }

  memSize() {
    return 0;
  }

  createTable<T extends ColMap>(name: string, colMap: T): Table<T> {
    const id = this.ops.createTable(name);
    return new Table<T>(colMap, id, this);
  }

  insertOnTable(
    tableID: number,
    col: number,
    row_id: number,
    value: unknown,
    tag: Something["tag"],
  ) {
    this.ops.putSomethingOnStack(value, tag);
    this.ops.tableInsert(tableID, col, row_id);
  }

  addListenerToRow(tableID: number, rowID: number, fn: () => void) {
    const result = this.ops.addListenerToRow(tableID, rowID);
    this.listeners.set(result, fn);
    return result;
  }

  notifyAll() {
    this.withLock(() => {
      const arr = this.ops.takeNotifications();
      arr.forEach((id) => {
        const listener = this.listeners.get(id);
        listener?.();
      });
    });
  }

  getFromTable(
    tableID: number,
    rowID: number,
    col: number,
  ): Something["value"] | null {
    this.ops.tableGetSomething(tableID, col, rowID);
    const value = popObjectFromStack();
    return value ?? null;
  }

  deleteRowFromTable(tableID: number, rowID: number): void {
    this.ops.deleteRowFromTable(tableID, rowID);
  }

  removeListenerFromRow(tableID: number, rowID: number, listenerID: number) {
    this.listeners.delete(listenerID);
    this.ops.removeListenerFromRow(tableID, rowID, listenerID);
  }

  getRowFromTable(tableID: number, rowID: number): Something["value"][] {
    this.ops.getRowFromTable(tableID, rowID);
    return getWholeStack();
  }

  createWorker(): WorkerData {
    this.workerID += 1;
    return {
      memory: this.memory,
      workerID: this.workerID,
    };
  }

  static i32(value: number): I32 {
    return { tag: "i32", value };
  }

  static f64(value: number): F64 {
    return { tag: "f64", value };
  }

  static blob(value: Uint8Array): Blob {
    return { tag: "blob", value };
  }

  static string(value: string): String {
    return { tag: "string", value };
  }

  static null(): Null {
    return { tag: "null", value: null };
  }

  static stack() {
    return jsStack;
  }

  static somethingFromValue(value: any): Something | null {
    if (typeof value === "number") {
      return AnyStore.f64(value);
    } else if (typeof value === "string") {
      return AnyStore.string(value);
    } else if (value === null) {
      return { tag: "null", value: null };
    } else if (value instanceof Uint8Array) {
      return AnyStore.blob(value);
    }
    return null;
  }
}

type ColMap = Record<string, Something["tag"]>;

type ValueMap = {
  i32: number;
  string: string;
  null: null;
  f64: number;
  blob: Uint8Array;
};

export class Table<T extends ColMap> {
  colMap: Map<string, number> = new Map();
  private rowConstructor: typeof _Row;
  constructor(
    private tags: T,
    private id: number,
    private wdb: AnyStore,
  ) {
    Object.keys(tags).forEach((colName, index) => {
      this.colMap.set(colName, index);
    });

    class ThisRow extends _Row<any> {}

    for (const col in this.tags) {
      const colIndex = this.colMap.get(col)!;
      const tag = this.tags[col];
      const set: any = new Function(
        "value",
        `
        this.table._insert(this.id, value, "${col}", "${tag}");
        return value;
        `,
      );

      const get: any = new Function(`
        if(!arguments.length) {
          if(this.cache) {
            return this.cache[${colIndex}];
          }
          return this.table.wdb.getFromTable(this.table.id, this.id, ${colIndex});
        }`);

      Object.defineProperties(ThisRow.prototype as any, {
        [col]: {
          configurable: true,
          get: get,
          set: set,
          enumerable: true,
        },
      });
    }

    this.rowConstructor = ThisRow;
  }

  private tagOf(colName: keyof T): Something["tag"] {
    return this.tags[colName];
  }

  addListenerToRow(rowID: number, fn: () => void) {
    return this.wdb.addListenerToRow(this.id, rowID, fn);
  }

  getRow(rowID: number): Something["value"][] {
    return this.wdb.getRowFromTable(this.id, rowID);
  }

  _insert(rowID: number, value: unknown, colName: keyof T) {
    const col = this.colMap.get(colName as string);
    this.wdb.insertOnTable(this.id, col!, rowID, value, this.tagOf(colName));
  }

  removeListenerFromRow(listenerID: number, rowID: number) {
    this.wdb.removeListenerFromRow(this.id, rowID, listenerID);
  }

  get(rowID: number, colName: keyof T): Something["value"] | null {
    const col = this.colMap.get(colName as string);
    return this.wdb.getFromTable(this.id, rowID, col!);
  }

  deleteRow(rowID: number) {
    this.wdb.deleteRowFromTable(this.id, rowID);
  }

  row(key: Something) {
    const id = this.wdb.createRow(key, this.id);
    return new this.rowConstructor<T>(this, id, key) as Row<T>;
  }
}

export type Row<T extends ColMap> = {
  [K in keyof T as `${K & string}`]: ValueMap[T[K]] | null;
} & _Row<T>;

export class _Row<T extends ColMap> {
  private cache: Something["value"][] | null = null;

  constructor(
    private table: Table<T>,
    private id: number = 0,
    public readonly key: Something,
  ) {}

  private load() {
    this.cache = this.table.getRow(this.id);
  }

  cached(onUpdate?: () => void) {
    return this.addListener(() => {
      this.load();
      onUpdate?.();
    });
  }

  addListener(fn: () => void) {
    return this.table.addListenerToRow(this.id, fn);
  }

  delete() {
    return this.table.deleteRow(this.id);
  }

  removeListener(listenerID: number) {
    this.table.removeListenerFromRow(listenerID, this.id);
  }

  getRow(): Something["value"][] {
    if (this.cache) {
      return this.cache;
    }
    return this.table.getRow(this.id);
  }
}

type I32 = { tag: "i32"; value: number };
type String = { tag: "string"; value: string };
type Null = { tag: "null"; value: null };
type F64 = { tag: "f64"; value: number };
type Blob = { tag: "blob"; value: Uint8Array };

export type Something = I32 | String | Null | F64 | Blob;

class Ops {
  constructor(private out: InitOutput) {
    this.exports.start();
  }

  get exports() {
    return this.out;
  }

  createTable(name: string) {
    this.putSomethingOnStack(name, "string");
    return this.exports.table_create();
  }

  getTableIDFromName(name: string): number | null {
    this.putSomethingOnStack(name, "string");
    const id = this.exports.table_get_id_from_name();
    return id === -1 ? null : id;
  }

  putSomethingOnStack(value: unknown, tag: Something["tag"]) {
    if (value == null) {
      this.pushNullToStack();
    } else if (tag === "i32") {
      this.somethingPushi32ToStack(value as number);
    } else if (tag === "string") {
      this.pushStringToStack(value as string);
    } else if (tag === "f64") {
      this.somethingPushf64ToStack(value as number);
    } else if (tag === "blob") {
      this.somethingPushBlobToStack(value as Uint8Array);
    } else if (tag === "null") {
      this.pushNullToStack();
    }
  }

  removeListenerFromRow(tableID: number, rowID: number, listenerID: number) {
    this.exports.table_remove_listener(tableID, listenerID, rowID);
  }

  deleteRowFromTable(tableID: number, rowID: number): void {
    this.exports.delete_row_from_table(tableID, rowID);
  }

  getRowFromTable(tableID: number, rowID: number): void {
    this.exports.table_get_row(tableID, rowID);
  }

  pushNullToStack(): void {
    this.exports.something_push_null_to_stack();
  }

  somethingPushBlobToStack(value: Uint8Array): void {
    pushBlobToStack(value);
    this.exports.something_push_blob();
  }

  lock() {
    this.exports.lock();
  }

  unlock() {
    this.exports.unlock();
  }

  somethingPushf64ToStack(value: number): void {
    this.exports.something_push_f64_to_stack(value);
  }

  takeNotifications(): number[] {
    this.exports.db_take_notifications();
    return getWholeStack();
  }

  addListenerToRow(tableID: number, rowID: number): number {
    return this.exports.table_add_listener_to_row(tableID, rowID);
  }

  somethingPushi32ToStack(value: number): void {
    this.exports.something_push_i32_to_stack(value);
  }

  pushStringToStack(str: string): void {
    pushToStringStack(str);
    this.exports.something_push_string();
  }

  tableInsert(tableID: number, col: number, rowID: number): void {
    this.exports.table_insert(tableID, col, rowID);
  }

  tableGetSomething(tableID: number, col: number, rowID: number): void {
    this.exports.table_get_something(tableID, col, rowID);
  }
}
