import fs from "fs";
import { WDB } from "./WDB";

async function main() {
  const data = fs.readFileSync(
    "../target/wasm32-unknown-unknown/release/any_store.wasm"
  );

  const wdb = await WDB.create(data);
  const table = wdb.createTable();
  const col = 0;
  const k1 = WDB.i64(123n);
  const v1 = WDB.string("Hello, World!");
  wdb.insertOnTable(table, col, k1, v1);
  const k2 = WDB.i64(456n);
  const v2 = WDB.i64(789n);
  wdb.insertOnTable(table, col, k2, v2);
  const val1 = wdb.getFromTable(table, k1, col);
  const val2 = wdb.getFromTable(table, k2, col);
  console.log("Got value for key 123:", val1);
  console.log("Got value for key 456:", val2);
}

main();
