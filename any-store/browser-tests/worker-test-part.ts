import { AnyStore } from "../src/WDB";

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
        const current = row.get("counter") as number;
        row.update("counter", AnyStore.i32(current + 1));
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
