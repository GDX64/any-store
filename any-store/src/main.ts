import Worker from "./test.worker?worker";
import { WDB } from "./WDB";
import wasmModule from "../../target/wasm32-unknown-unknown/release/any_store.wasm?url";

async function startWorkerTest() {
  const val = new Worker();
  val.onmessage = (e) => {
    console.log("Message from worker:", e.data);
  };

  const response = await fetch(wasmModule);
  const data = await response.arrayBuffer();
  const db = await WDB.create(data);
  const table = db.createTable({
    age: "i32",
  });
  table.insert(WDB.i32(1), WDB.i32(25), "age");
  const row = table.row(WDB.i32(1));
  console.log("Main -> Row age:", row.get("age"));

  val.postMessage({
    module: db.getModule(),
    memory: db.getMemory(),
  });
}

startWorkerTest();

