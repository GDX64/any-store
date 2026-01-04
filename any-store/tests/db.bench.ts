import { bench, describe } from "vitest";
import { WDB } from "../src/WDB";
import fs from "fs";
const wasmPath = "../target/wasm32-unknown-unknown/release/any_store.wasm";

describe("benchmarks", async () => {
  const data = fs.readFileSync(wasmPath);
  const mockData = Array.from({ length: 2000 }, (_, i) => {
    return {
      age: Math.round(Math.random() * 100),
      height: Math.random() * 2,
      name: `Name_${i}`,
    };
  });
  let count = 0;
  const db = await WDB.create(data);
  const table = db.createTable({
    name: "string",
    age: "i32",
    height: "f64",
  });

  bench(
    "insert on db",
    async () => {
      try {
        if (count > 1) {
          return;
        }
        count += 1;
        mockData.forEach((item, index) => {
          const key = WDB.i32(index);
          table.insert(key, WDB.string(item.name), "name");
          table.insert(key, WDB.i32(item.age), "age");
          table.insert(key, WDB.f64(item.height), "height");
        });
      } catch (e) {
        console.error(e);
      }
    },
    {
      setup() {},
    }
  );

  bench("insert on hashmap", () => {
    const map = new Map<
      number,
      { name: string; age: number; height: number }
    >();
    mockData.forEach((item, index) => {
      map.set(index, { name: item.name, age: item.age, height: item.height });
    });
  });
});
