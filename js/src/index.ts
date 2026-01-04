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
  table.insert("name", k1, WDB.string("Alice"));
  table.insert("age", k1, WDB.i32(30));
  table.insert("height", k1, WDB.f64(1.75));
  const val1 = table.get("name", k1);
  const val2 = table.get("age", k1);
  const val3 = table.get("height", k1);
  console.log("Got value for key 123:", val1, val2, val3);
}

main();
