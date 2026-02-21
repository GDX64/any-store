import { describe, expect, test } from "vitest";
import Worker from "./worker-test-part?worker";
import { WDB } from "../src/WDB";

describe("Web Worker", async () => {
  test("counter", async () => {
    function workerWrapper() {
      const val = new Worker();

      function waitNextMessage() {
        return new Promise((resolve) => {
          val.onmessage = resolve;
        });
      }
      val.postMessage(db.createWorker());

      return {
        channel: val,
        waitNextMessage,
      };
    }

    const db = await WDB.create(0);
    const row = db.withLock(() => {
      const table = db.createTable({
        counter: "i32",
      });

      const row = table.row(WDB.i32(1));
      row.update("counter", WDB.i32(0));
      return row;
    });

    const w1 = workerWrapper();
    const w2 = workerWrapper();
    // const w3 = workerWrapper();
    // const w4 = workerWrapper();

    const allFinished = Promise.all([
      w1.waitNextMessage(),
      w2.waitNextMessage(),
      // w3.waitNextMessage(),
      // w4.waitNextMessage(),
    ]);

    // for (let i = 0; i < 1000_000; i++) {
    //   db.withLock(() => {
    //     const current = row.get("counter") as number;
    //     row.update("counter", WDB.i32(current + 1));
    //   });
    // }

    await allFinished;

    expect(row.get("counter")).toBe(2_000_000);
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
