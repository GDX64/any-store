import { AnyStore } from "@glmachado/any-store";

const db = await AnyStore.create();
const table = db.createTable(
  {
    counter: "i32",
  },
  "hello",
);
const row = table.row(AnyStore.i32(1));

console.log(row.get("counter")); // null

row.update("counter", AnyStore.i32(0));

console.log(row.get("counter")); // 0
