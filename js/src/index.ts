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
  });
  const k1 = WDB.i32(123);
  const v1 = WDB.string("Hello, World!");
  const v2 = WDB.i32(789);
  table.insert("name", k1, v1);
  table.insert("age", k1, v2);
  const val1 = table.get("name", k1);
  const val2 = table.get("age", k1);
  console.log("Got value for key 123:", val1, val2);
}

main();
