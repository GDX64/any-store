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
});

// Get a row handle
const row = table.row(AnyStore.i32(1));

// Update values
row.update("name", AnyStore.string("Alice"));
row.update("age", AnyStore.i32(30));
row.update("height", AnyStore.f64(1.75));

// Read values
console.log(row.get("name")); // "Alice"
console.log(row.get("age")); // 30
console.log(row.get("height")); // 1.75

// Delete a row
row.delete();
console.log(row.get("name")); // null
```

## Data Types

The library supports the following data types:

- `AnyStore.i32(number)` - 32-bit integer
- `AnyStore.f64(number)` - 64-bit float
- `AnyStore.string(string)` - UTF-8 string
- `AnyStore.blob(Uint8Array)` - Binary data
- `AnyStore.null()` - Null value

## Table Operations

```ts
const table = db.createTable("products", {
  name: "string",
  price: "f64",
  stock: "i32",
});

// Insert individual columns
const key = AnyStore.i32(1);
table.insert(key, AnyStore.string("Laptop"), "name");
table.insert(key, AnyStore.f64(999.99), "price");
table.insert(key, AnyStore.i32(5), "stock");

// Insert a complete row at once
table.insertRow(
  AnyStore.i32(2),
  [
    AnyStore.string("Mouse"),
    AnyStore.f64(29.99),
    AnyStore.i32(100),
  ]
);

// Get a single value
const price = table.get(key, "price"); // 999.99

// Get the entire row
const row = table.getRow(key); // ["Laptop", 999.99, 5]

// Delete a row
table.deleteRow(key);
```

## Reactive Updates with Listeners

```ts
const table = db.createTable("counter", { value: "i32" });
const row = table.row(AnyStore.i32(1));

// Add a listener to be notified when the row changes
const listenerID = row.addListener(() => {
  console.log("Row changed:", row.get("value"));
});

// Update triggers the listener on next notifyAll()
row.update("value", AnyStore.i32(10));
db.notifyAll(); // Triggers all listeners

// Remove the listener
row.removeListener(listenerID);
```

## Cached Rows

Use cached rows for better performance when reading values frequently:

```ts
const row = table.row(AnyStore.i32(1));

// Enable caching with optional update callback
row.cached(() => {
  console.log("Cache updated:", row.get("value"));
});

row.update("value", AnyStore.i32(10));
console.log(row.get("value")); // Still old value until notified

db.notifyAll(); // Updates cache and triggers callback
console.log(row.get("value")); // 10 (from cache)
```

## Usage with Workers

Share the database across multiple threads without serialization:

```ts
// Main thread
import { AnyStore } from "@glmachado/any-store";

const db = await AnyStore.create(0); // Worker ID 0 for main thread
const table = db.createTable("shared", { counter: "i32" });

const row = table.row(AnyStore.i32(1));
row.update("counter", AnyStore.i32(0));

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
    const current = row.get("counter") as number;
    row.update("counter", AnyStore.i32(current + 1));
  });
};
```

## Thread Safety

Use `withLock()` to ensure atomic operations when accessing shared data:

```ts
db.withLock(() => {
  const current = row.get("counter") as number;
  row.update("counter", AnyStore.i32(current + 1));
});
```

## API Reference

### AnyStore

- `static create(id?: number, bufferSource?: BufferSource): Promise<AnyStore>` - Create a new database
- `static fromModule(workerData: WorkerData): Promise<AnyStore>` - Create from worker data
- `createTable<T>(name: string, colMap: T): Table<T>` - Create a new table
- `getTable<T>(name: string, colMap: T): Table<T> | null` - Get an existing table by name
- `createWorker(): WorkerData` - Create worker data for sharing
- `withLock<T>(fn: () => T): T` - Execute function with lock
- `notifyAll(): void` - Notify all listeners
- `memSize(): number` - Get memory size in pages

### Table

- `row(key: Something): Row` - Get a row handle
- `insert(key: Something, value: Something, colName: string): void` - Insert a value
- `insertRow(key: Something, values: Something[]): void` - Insert a complete row
- `get(key: Something, colName: string): any` - Get a value
- `getRow(key: Something): any[]` - Get all values in a row
- `deleteRow(key: Something): void` - Delete a row
- `addListenerToRow(key: Something, fn: () => void): number` - Add a listener
- `removeListenerFromRow(key: Something, listenerID: number): void` - Remove a listener

### Row

- `get(colName: string): any` - Get a column value
- `update(colName: string, value: Something): void` - Update a column
- `delete(): void` - Delete the row
- `getRow(): any[]` - Get all values
- `addListener(fn: () => void): number` - Add a listener
- `cached(onUpdate?: () => void): number` - Enable caching
- `removeListener(listenerID: number): void` - Remove a listener