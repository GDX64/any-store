import { describe, expect, test } from "vitest";
import Worker from "./worker-test-part?worker";
import { AnyStore } from "../src/AnyStore";

describe("Web Worker", async () => {
  test("counter", async () => {
    const N = 50_000;
    const numWorkers = 6;
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

    const db = await AnyStore.create();
    const row = db.withLock(() => {
      const table = db.createTable("hello", {
        counter: "i32",
      });

      const row = table.row(AnyStore.i32(1));
      row.counter = 0;
      return row;
    });

    const workers = Array.from({ length: numWorkers }, workerWrapper);

    for (let i = 0; i < N; i++) {
      await db.withLockAsync(async () => {
        const current = row.counter ?? 0;
        row.counter = current + 1;
      });
    }

    const allFinished = await Promise.all(
      workers.map((w) => w.waitNextMessage()),
    );

    if (allFinished.some((msg) => !msg.data.ok)) {
      throw new Error("One of the workers failed");
    }

    expect(row.counter).toBe(N * (allFinished.length + 1));
  });
});
