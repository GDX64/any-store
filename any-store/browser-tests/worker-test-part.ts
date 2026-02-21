import { WDB } from "../src/WDB";

self.onmessage = async (event) => {
  console.log("Message from main thread:", event.data);
  const db = await WDB.fromModule(event.data);
  const row = db.withLock(() => {
    const table = db.getTable(1, { counter: "i32" });
    const row = table.row(WDB.i32(1));
    return row;
  });

  for (let i = 0; i < 1000_000; i++) {
    db.withLock(() => {
      const current = row.get("counter") as number;
      row.update("counter", WDB.i32(current + 1));
    });
  }
  self.postMessage({
    data: "worker is done",
  });
};
