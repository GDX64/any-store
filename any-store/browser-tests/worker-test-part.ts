import { WDB } from "../src/WDB";

self.onmessage = async (event) => {
  console.log("Message from main thread:", event.data);
  const module = event.data.module;
  const memory = event.data.memory;
  const db = await WDB.fromModule(module, memory, 1);
  const table = db.getTable(1, { counter: "i32" });
  const row = table.row(WDB.i32(1));
  self.postMessage({
    data: "worker is ready",
  });

  for (let i = 0; i < 100_000; i++) {
    const current = row.get("counter") as number;
    row.update("counter", WDB.i32(current + 1));
    db.commit();
  }
  self.postMessage({
    data: "worker is done",
  });
};
