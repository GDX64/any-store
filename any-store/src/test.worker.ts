import { WDB } from "./WDB";

console.log("Worker started");

self.onmessage = async (event) => {
  console.log("Message from main thread:", event.data);
  const module = event.data.module;
  const memory = event.data.memory;
  const db = await WDB.fromModule(module, memory);
  const table = db.getTable(1, { age: "i32" });
  const row = table.row(WDB.i32(1));
  const age = row.get("age");

  console.log("Worker -> Row age:", age);
};
