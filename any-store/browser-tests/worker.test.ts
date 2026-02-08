import { describe, test } from "vitest";
import Worker from "./worker-test-part?worker";
import { WDB } from "../src/WDB";
import wasmModule from "../../target/wasm32-unknown-unknown/release/any_store.wasm?url";

describe("Web Worker", async () => {
  test("counter", async () => {
    const val = new Worker();

    function waitNextMessage() {
      return new Promise((resolve) => {
        val.onmessage = resolve;
      });
    }

    const response = await fetch(wasmModule);
    const data = await response.arrayBuffer();
    const db = await WDB.create(data, 0);
    const table = db.createTable({
      counter: "i32",
    });

    const row = table.row(WDB.i32(1));
    row.update("counter", WDB.i32(0));
    db.commit();
    val.postMessage({
      module: db.getModule(),
      memory: db.getMemory(),
    });

    await waitNextMessage();

    for (let i = 0; i < 1000; i++) {
      const current = row.get("counter") as number;
      console.log("Main -> Current counter:", current);
      row.update("counter", WDB.i32(current + 1));
      db.commit();
    }

    await waitNextMessage();

    console.log("Main -> Row counter:", row.get("counter"));
  });
});

// function insertMockData(table: Table<any>, db: WDB) {
//   const mockData = Array.from({ length: 10_000 }, (_, i) => {
//     return {
//       age: Math.round(Math.random() * 100),
//       height: Math.random() * 2,
//       weight: Math.random() * 100,
//     };
//   });

//   mockData.forEach((item, index) => {
//     const key = WDB.i32(index);
//     table.insertRow(key, [
//       WDB.f64(item.weight),
//       WDB.i32(item.age),
//       WDB.f64(item.height),
//     ]);
//     db.commit();
//   });
// }
