import { WDB } from "../src/WDB";

self.onmessage = async (event) => {
  console.log("Message from main thread:", event.data);
  const db = await WDB.fromModule(event.data.dbData);
  const row = db.withLock(() => {
    const id = db.tableIDFromName("hello");
    if (!id) {
      throw new Error("Table 'hello' not found");
    }
    console.log("Got table ID:", id);
    const table = db.getTable(id, { counter: "i32" });
    const row = table.row(WDB.i32(1));
    return row;
  });
  const n = event.data.n as number;
  try {
    for (let i = 0; i < n; i++) {
      db.withLock(() => {
        const current = row.get("counter") as number;
        row.update("counter", WDB.i32(current + 1));
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
