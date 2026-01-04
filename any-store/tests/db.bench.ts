import { bench, describe } from "vitest";
import { WDB } from "../src/WDB";
import { DatabaseSync } from "node:sqlite";
import fs from "fs";
const wasmPath = "../target/wasm32-unknown-unknown/release/any_store.wasm";

describe("benchmarks inserts", async () => {
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
        map.set(
          index,
          structuredClone({
            name: item.name.value,
            age: item.age.value,
            height: item.height.value,
          })
        );
      });
    },
    {
      time: 100,
    }
  );

  const sqliteDB = new DatabaseSync(":memory:");
  let count = 0;
  bench(
    "insert on sqlite",
    () => {
      count++;
      const tableName = `test_${count}`;
      sqliteDB.exec(`pragma journal_mode = WAL;`);
      sqliteDB.exec(
        `CREATE TABLE ${tableName} (id INTEGER PRIMARY KEY, name TEXT, age INTEGER, height REAL);`
      );
      // Wrap all inserts in a single transaction
      sqliteDB.exec("BEGIN TRANSACTION");
      const stmt = sqliteDB.prepare(
        `INSERT INTO ${tableName} (id, name, age, height) VALUES (?, ?, ?, ?);`
      );
      mockData.forEach((item, index) => {
        stmt.run(index, item.name.value, item.age.value, item.height.value);
      });
      sqliteDB.exec("COMMIT");
    },
    {
      time: 100,
    }
  );
});
