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

// Create or get a row handle with a key
const row = table.createRow(AnyStore.i32(1));

// Set values using property accessors (fully type-safe)
row.name = "Alice";
row.age = 30;
row.height = 1.75;
row.data = new Uint8Array([1, 2, 3]);

// Read values using property accessors
const name: string | null = row.name; // "Alice"
const age: number | null = row.age; // 30
const height: number | null = row.height; // 1.75
const data: Uint8Array | null = row.data; // Uint8Array([1, 2, 3])

// Set to null to remove a value
row.name = null;
console.log(row.name); // null

// Delete the entire row
row.delete();
console.log(row.age); // null
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

// Create or get a row handle
const row = table.createRow(AnyStore.i32(1));

// Set individual columns using property accessors
row.name = "Laptop";
row.price = 999.99;
row.stock = 5;

// Get individual values using property accessors
const price = row.price; // 999.99

// Get the entire row as an array (in schema order)
const rowData = row.getRow(); // ["Laptop", 999.99, 5]

// Use destructuring for convenient access
const { name, price: currentPrice, stock } = row;

// Delete the row
row.delete();
```

## Reactive Updates with Listeners

Listen to row changes for reactive updates:

```ts
const table = db.createTable("counter", { value: "i32" });
const row = table.createRow(AnyStore.i32(1));

// Add a listener to be notified when the row changes
const listenerID = row.addListener(() => {
  console.log("Row changed:", row.value);
});

// Listeners are not called immediately after updates
row.value = 10;

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
const row = table.createRow(AnyStore.i32(1));

// Enable caching with optional update callback
row.cached(() => {
  console.log("Cache updated:", row.value);
});

// First read returns null (no data yet)
console.log(row.value); // null

row.value = 0;

// Cache is not updated immediately
console.log(row.value); // null (still using cached value)

// Notify to update the cache
db.notifyAll(); // Triggers callback and updates cache
console.log(row.value); // 0 (from cache)

row.value = 1;

// Cache still shows old value until notified
console.log(row.value); // 0

db.notifyAll(); // Updates cache again
console.log(row.value); // 1
```

## Usage with Workers

Share the database across multiple threads without serialization:

```ts
// Main thread
import { AnyStore } from "@glmachado/any-store";

const db = await AnyStore.create();
const table = db.createTable("shared", { counter: "i32" });

const row = table.createRow(AnyStore.i32(1));
row.counter = 0;

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
  
  const row = table.createRow(AnyStore.i32(1));
  
  // Read and modify shared data
  db.withLock(() => {
    const current = row.counter ?? 0;
    row.counter = current + 1;
  });
};
```

## Thread Safety

When working with workers, use `withLock()` to ensure atomic operations:

```ts
// Synchronous lock - blocks in workers (uses Atomics.wait)
// Spins in main thread (cannot use Atomics.wait)
db.withLock(() => {
  const current = row.counter ?? 0;
  row.counter = current + 1;
});

// Async lock - doesn't block main thread
await db.withLockAsync(async () => {
  const current = row.counter ?? 0;
  row.counter = current + 1;
});
```

## Atomic Operations on Rows

Perform multiple operations on a single row atomically using `withLock()`:

```ts
const table = db.createTable("counter", { value: "i32" });
const row = table.createRow(AnyStore.i32(1));

db.withLock(() => {
  row.value = 0;
  for (let i = 0; i < 1000_000; i++) {
    row.value += 1;
  }
});

console.log(row.value); // 1000000
```

## Foreign Keys and Querying

Use the `where()` method to query rows by column value, useful for foreign key relationships:

```ts
const db = await AnyStore.create();
const people = db.createTable("people", {
  name: "string",
  team: "i32",
});
const teams = db.createTable("teams", {
  name: "string",
});

// Create a team
const team1 = teams.createRow(AnyStore.i32(1));
team1.name = "Team A";

// Create people with foreign key reference to team
const person1 = people.createRow(AnyStore.i32(1));
person1.name = "Alice";
person1.team = team1.rowID; // Use rowID as foreign key

const person2 = people.createRow(AnyStore.i32(2));
person2.name = "Bob";
person2.team = team1.rowID;

// Query all people in Team A
const teamMembers = people.where("team", team1.rowID);
console.log(teamMembers); // [1, 2] - array of row IDs
```

## Clearing Tables

Remove all rows from a table using `clear()`:

```ts
const table = db.createTable("temp", { value: "i32" });

// Add some data
table.createRow(AnyStore.i32(1)).value = 100;
table.createRow(AnyStore.i32(2)).value = 200;

// Clear all rows
table.clear();

// Check if rows exist
const row = table.getRow(AnyStore.i32(1));
console.log(row); // null
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
- `memSize(): number` - Get current memory size in bytes

### Table<T>

**Methods**
- `createRow(key: Something): Row<T>` - Create or get a row handle with the given key
- `getRow(key: Something): Row<T> | null` - Get a row if it exists, return null otherwise
- `where<K>(colName: K, value: ValueMap[T[K]]): number[]` - Query rows by column value, returns array of row IDs
- `clear(): void` - Remove all rows from the table

**Note:** Most table operations should be done through `Row` objects rather than directly on the table.

### Row<T>

**Properties**
- `rowKey: Something` - The row's key (readonly)
- `rowID: number` - The row's internal ID, useful for foreign key references
- `[columnName]: ValueType | null` - Dynamic properties for each column defined in schema (type-safe)

**Methods**
- `getRow(): any[]` - Get entire row as array in schema order
- `delete(): void` - Delete the entire row
- `addListener(fn: () => void): number` - Add listener, returns listener ID
- `removeListener(listenerID: number): void` - Remove listener by ID
- `cached(onUpdate?: () => void): number` - Enable caching mode with optional callback, returns listener ID

**Property Accessors**

Rows have type-safe property accessors for all columns. You can read and write values directly:

```ts
const table = db.createTable("users", {
  name: "string",
  age: "i32",
});

const row = table.createRow(AnyStore.i32(1));

// Set values
row.name = "Alice";
row.age = 30;

// Get values
const name = row.name; // string | null
const age = row.age;   // number | null

// Use destructuring
const { name: userName, age: userAge } = row;
```