import { bench, describe } from "vitest";
import { WDB } from "../src/WDB";
import fs from "fs";
const wasmPath = "../target/wasm32-unknown-unknown/release/any_store.wasm";

describe("benchmarks", async () => {
  const data = fs.readFileSync(wasmPath);
  const mockData = Array.from({ length: 10_000 }, (_, i) => {
    return {
      age: WDB.i32(Math.round(Math.random() * 100)),
      height: WDB.f64(Math.random() * 2),
      name: WDB.string("PETR4" + i),
    };
  });
  const db = await WDB.create(data);

  bench(
    "insert on db",
    async () => {
      try {
        const table = db.createTable({
          name: "string",
          age: "i32",
          height: "f64",
        });
        mockData.forEach((item, index) => {
          const key = WDB.i32(index);
          table.insert(key, item.name, "name");
          table.insert(key, item.age, "age");
          table.insert(key, item.height, "height");
        });
      } catch (e) {
        console.error(e);
      }
    },
    {
      time: 100,
    }
  );

  bench(
    "insert on hashmap",
    () => {
      const map = new Map<number, any>();
      mockData.forEach((item, index) => {
        map.set(index, {
          name: item.name.value,
          age: item.age.value,
          height: item.height.value,
        });
      });
    },
    {
      time: 100,
    }
  );
});
