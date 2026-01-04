import fs from "fs";
import { WDB } from "./WDB";

async function main() {
  const data = fs.readFileSync(
    "../target/wasm32-unknown-unknown/release/any_store.wasm"
  );

  const wdb = await WDB.create(data);
  const table = wdb.createTable({
    name: "string",
    age: "i32",
    height: "f64",
  });
  const k1 = WDB.i32(123);
  table.insert(k1, WDB.string("Alice"), "name");
  table.insert(k1, WDB.i32(30), "age");
  table.insert(k1, WDB.f64(1.75), "height");
  table.insert(WDB.i32(0), WDB.string("Bob"), "name");
  const row1 = table.row(k1);
  const val1 = row1.get("name");
  const val2 = row1.get("age");
  const val3 = row1.get("height");
  console.log("Got value for key 123:", val1, val2, val3);
  console.log("Got value for key 0:", table.get(WDB.i32(0), "name"));
}

main();
