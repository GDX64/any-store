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
