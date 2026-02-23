import { describe, expect, test } from "vitest";
import Worker from "./worker-test-part?worker";
import { WDB } from "../src/WDB";

describe("Web Worker", async () => {
  test("counter", async () => {
    const N = 100_000;
    const numWorkers = navigator.hardwareConcurrency - 1;
    function workerWrapper() {
      const val = new Worker();

      function waitNextMessage() {
        return new Promise<any>((resolve) => {
          val.onmessage = resolve;
        });
      }
      val.postMessage({ dbData: db.createWorker(), n: N });

      return {
        channel: val,
        waitNextMessage,
      };
    }

    const db = await WDB.create(0);
    const row = db.withLock(() => {
      const table = db.createTable(
        {
          counter: "i32",
        },
        "hello",
      );

      const row = table.row(WDB.i32(1));
      row.update("counter", WDB.i32(0));
      return row;
    });

    const workers = Array.from({ length: numWorkers }, workerWrapper);

    for (let i = 0; i < N; i++) {
      db.withLock(() => {
        const current = row.get("counter") as number;
        row.update("counter", WDB.i32(current + 1));
      });
    }

    const allFinished = await Promise.all(
      workers.map((w) => w.waitNextMessage()),
    );

    if (allFinished.some((msg) => !msg.data.ok)) {
      throw new Error("One of the workers failed");
    }

    expect(row.get("counter")).toBe(N * (allFinished.length + 1));
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
