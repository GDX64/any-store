## Any Store

A simple in-memory, embedded database with key-value storage. Share data between different threads and workers without serialization costs. Data can be written and read from worker threads and the main thread using shared WebAssembly memory.

## Installation

```bash
npm install @glmachado/any-store
```

## Basic Usage

```ts
import { AnyStore } from "@glmachado/any-store";

// Create a database
const db = await AnyStore.create();

// Create a table with name and schema
const table = db.createTable("users", {
  name: "string",
  age: "i32",
  height: "f64",
  data: "blob",
});

// Get a row handle with a key
const row = table.row(AnyStore.i32(1));

// Update values (values are passed directly, not wrapped)
row.update("name", "Alice");
row.update("age", 30);
row.update("height", 1.75);
row.update("data", new Uint8Array([1, 2, 3]));

// Read values (fully type-safe)
const name: string | null = row.get("name"); // "Alice"
const age: number | null = row.get("age"); // 30
const height: number | null = row.get("height"); // 1.75
const data: Uint8Array | null = row.get("data"); // Uint8Array([1, 2, 3])

// Update with null to remove a value
row.update("name", null);
console.log(row.get("name")); // null

// Delete the entire row
row.delete();
console.log(row.get("age")); // null
```

## Key Types

Keys must be created using these helper functions:

- `AnyStore.i32(number)` - 32-bit integer key
- `AnyStore.f64(number)` - 64-bit float key
- `AnyStore.string(string)` - String key
- `AnyStore.blob(Uint8Array)` - Binary key

## Column Types

When defining table schemas, use these type names:

- `"i32"` - 32-bit integer values
- `"f64"` - 64-bit float values
- `"string"` - UTF-8 string values
- `"blob"` - Binary data (Uint8Array) values

## Working with Rows

```ts
const table = db.createTable("products", {
  name: "string",
  price: "f64",
  stock: "i32",
});

// Create a row handle
const row = table.row(AnyStore.i32(1));

// Update individual columns
row.update("name", "Laptop");
row.update("price", 999.99);
row.update("stock", 5);

// Get individual values
const price = row.get("price"); // 999.99

// Get the entire row as an array
const rowData = row.getRow(); // ["Laptop", 999.99, 5]

// Delete the row
row.delete();
```

## Reactive Updates with Listeners

Listen to row changes for reactive updates:

```ts
const table = db.createTable("counter", { value: "i32" });
const row = table.row(AnyStore.i32(1));

// Add a listener to be notified when the row changes
const listenerID = row.addListener(() => {
  console.log("Row changed:", row.get("value"));
});

// Listeners are not called immediately after updates
row.update("value", 10);

// Listeners trigger only when notifyAll() is called
db.notifyAll(); // Triggers all pending listener notifications

// Multiple notifyAll() calls only trigger each listener once per change
db.notifyAll(); // Listeners won't fire again unless row changes

// Remove the listener when done
row.removeListener(listenerID);
```

## Cached Rows for Performance

Enable caching on a row to avoid reading from the database on every access:

```ts
const table = db.createTable("counter", { value: "i32" });
const row = table.row(AnyStore.i32(1));

// Enable caching with optional update callback
row.cached(() => {
  console.log("Cache updated:", row.get("value"));
});

// First read returns null (no data yet)
console.log(row.get("value")); // null

row.update("value", 0);

// Cache is not updated immediately
console.log(row.get("value")); // null (still using cached value)

// Notify to update the cache
db.notifyAll(); // Triggers callback and updates cache
console.log(row.get("value")); // 0 (from cache)

row.update("value", 1);

// Cache still shows old value until notified
console.log(row.get("value")); // 0

db.notifyAll(); // Updates cache again
console.log(row.get("value")); // 1
```

## Usage with Workers

Share the database across multiple threads without serialization:

```ts
// Main thread
import { AnyStore } from "@glmachado/any-store";

const db = await AnyStore.create();
const table = db.createTable("shared", { counter: "i32" });

const row = table.row(AnyStore.i32(1));
row.update("counter", 0);

// Create worker data to share
const workerData = db.createWorker();

const worker = new Worker("./worker.js");
worker.postMessage(workerData);
```

```ts
// Worker thread (worker.js)
import { AnyStore } from "@glmachado/any-store";

self.onmessage = async (event) => {
  const workerData = event.data;
  const db = await AnyStore.fromModule(workerData);
  
  // Access the table created in the main thread
  const table = db.getTable("shared", { counter: "i32" });
  if (!table) {
    throw new Error("Table not found");
  }
  
  const row = table.row(AnyStore.i32(1));
  
  // Read and modify shared data
  db.withLock(() => {
    const current = row.get("counter") ?? 0;
    row.update("counter", current + 1);
  });
};
```

## Thread Safety

When working with workers, use `withLock()` to ensure atomic operations:

```ts
// Synchronous lock - blocks in workers (uses Atomics.wait)
// Spins in main thread (cannot use Atomics.wait)
db.withLock(() => {
  const current = row.get("counter") ?? 0;
  row.update("counter", current + 1);
});

// Async lock - doesn't block main thread
await db.withLockAsync(async () => {
  const current = row.get("counter") ?? 0;
  row.update("counter", current + 1);
});
```

## API Reference

### AnyStore

**Static Methods**
- `static create(): Promise<AnyStore>` - Create a new database instance
- `static fromModule(workerData: WorkerData): Promise<AnyStore>` - Create database from worker data
- `static i32(value: number): Something` - Create an i32 key/value
- `static f64(value: number): Something` - Create an f64 key/value
- `static string(value: string): Something` - Create a string key/value
- `static blob(value: Uint8Array): Something` - Create a blob key/value
- `static null(): Something` - Create a null value

**Instance Methods**
- `createTable<T>(name: string, colMap: T): Table<T>` - Create a new table with schema
- `getTable<T>(name: string, colMap: T): Table<T> | null` - Get existing table by name
- `createWorker(): WorkerData` - Create worker data for sharing across threads
- `withLock<T>(fn: () => T): T` - Execute function with exclusive lock (blocks in workers)
- `withLockAsync<T>(fn: () => Promise<T>): Promise<T>` - Execute function with exclusive lock (async)
- `notifyAll(): void` - Trigger all pending listener notifications
- `memSize(): number` - Get current memory size

### Table<T>

- `row(key: Something): Row<T>` - Get or create a row handle with the given key
- `get(rowID: number, colName: keyof T): any | null` - Get a value directly (internal use)
- `getRow(rowID: number): any[]` - Get entire row as array (internal use)
- `deleteRow(rowID: number): void` - Delete a row (internal use)
- `addListenerToRow(rowID: number, fn: () => void): number` - Add listener (internal use)
- `removeListenerFromRow(listenerID: number, rowID: number): void` - Remove listener (internal use)

**Note:** Most table operations should be done through `Row` objects rather than directly on the table.

### Row<T>

**Properties**
- `key: Something` - The row's key (readonly)

**Methods**
- `get<K extends keyof T>(colName: K): ValueType | null` - Get column value (type-safe)
- `update<K extends keyof T>(colName: K, value: ValueType | null): void` - Update column value
- `getRow(): any[]` - Get entire row as array in schema order
- `delete(): void` - Delete the entire row
- `addListener(fn: () => void): number` - Add listener, returns listener ID
- `removeListener(listenerID: number): void` - Remove listener by ID
- `cached(onUpdate?: () => void): number` - Enable caching mode with optional callback