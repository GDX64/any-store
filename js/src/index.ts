import fs from "fs";
import { WDB } from "./WDB";

async function main() {
  const data = fs.readFileSync(
    "../target/wasm32-unknown-unknown/release/any_store.wasm"
  );

  const wdb = await WDB.create(data);
  const table = wdb.createTable({
    name: "string",
  });
  const k1 = WDB.i64(123n);
  const v1 = WDB.string("Hello, World!");
  table.insert("name", k1, v1);
  const k2 = WDB.i64(456n);
  const v2 = WDB.i64(789n);
  table.insert("name", k2, v2);
  const val1 = table.get("name", k1);
  const val2 = table.get("name", k2);
  console.log("Got value for key 123:", val1);
  console.log("Got value for key 456:", val2);
}

main();
