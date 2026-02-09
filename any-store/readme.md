## Any Store

This package provides a simple in memory, embedded database with key-value storage. The main advantage of using this database is that with it you can share data between different threads and workers without the cost of for serialization. Data can be written and read in worker threads and the main thread.

### Usage

```ts
import { WDB } from "@glmachado/any-store";

const db = await WDB.create();
const table = db.createTable({
  counter: "i32",
});
const row = table.row(WDB.i32(1));

console.log(row.get("counter")); // null

row.update("counter", WDB.i32(0));
db.commit();

console.log(row.get("counter")); // 0
```

## Usage with workers

```ts
// in main thread
import { WDB } from "@glmachado/any-store";

const db = await WDB.create();
const table = db.createTable({
  counter: "i32",
});
const row = table.row(WDB.i32(1));
row.update("counter", WDB.i32(10));
db.commit();

const worker = new Worker("./example.worker.js");
worker.postMessage({
  module: db.getModule(),
  memory: db.getMemory(),
});
```

```ts
// in worker thread
import { WDB } from "@glmachado/any-store";

self.onmessage = async (event) => {
  const module = event.data.module;
  const memory = event.data.memory;
  const db = await WDB.fromModule(module, memory, 1);
  
  //this will read data from the table created in the main thread
  const table = db.getTable(1, { counter: "i32" });
  const row = table.row(WDB.i32(1));
  console.log(row.get("counter")); // 10
};
```