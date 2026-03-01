import { AnyStore } from "../src/AnyStore";

self.onmessage = async (event) => {
  console.log("Message from main thread:", event.data);
  const db = await AnyStore.fromModule(event.data.dbData);
  const row = db.withLock(() => {
    const table = db.getTable("hello", { counter: "i32" });
    if (!table) {
      throw new Error("Table 'hello' not found in worker");
    }
    const row = table.row(AnyStore.i32(1));
    return row;
  });
  const n = event.data.n as number;
  try {
    for (let i = 0; i < n; i++) {
      db.withLock(() => {
        const current = row.counter ?? 0;
        row.counter = current + 1;
      });
    }
    self.postMessage({
      ok: true,
    });
  } catch (e) {
    self.postMessage({
      ok: false,
    });
    console.error("Error in worker:", e);
  }
};
