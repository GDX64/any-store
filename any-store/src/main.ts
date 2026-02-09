import Worker from "./test.worker?worker";
import { WDB } from "./WDB";

async function startWorkerTest() {
  const val = new Worker();
  val.onmessage = (e) => {
    console.log("Message from worker:", e.data);
  };

  const db = await WDB.create(0);
  const table = db.createTable({
    weight: "f64",
    age: "i32",
    height: "f64",
  });
  table.insert(WDB.i32(1), WDB.i32(25), "age");
  db.commit();
  const row = table.row(WDB.i32(1));
  console.log("Main -> Row age:", row.get("age"));

  val.postMessage({
    module: db.getModule(),
    memory: db.getMemory(),
  });

  const mockData = Array.from({ length: 100_000 }, (_, i) => {
    return {
      age: Math.round(Math.random() * 100),
      height: Math.random() * 2,
      weight: Math.random() * 100,
    };
  });

  window.addEventListener("click", () => {
    mockData.forEach((item, index) => {
      const key = WDB.i32(index);
      table.insertRow(key, [
        WDB.f64(item.weight),
        WDB.i32(item.age),
        WDB.f64(item.height),
      ]);
    });

    setTimeout(() => {
      db.commit();
    }, 1000);
  });
}

startWorkerTest();
