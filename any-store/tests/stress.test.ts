import { setupFetch } from "./setupFetch";
import { AnyStore } from "../src/AnyStore";
import { expect, test, describe } from "vitest";

setupFetch();

describe("stress test", () => {
  test("stress memory", async () => {
    const mockData = new Map<
      number,
      { age: number; height: number; name: string }
    >();
    Array.from({ length: 1000 }, (_, i) => {
      mockData.set(i, {
        age: Math.round(Math.random() * 100),
        height: Math.random() * 2,
        name: `Name_${i}`,
      });
    });

    const N_REPETITIONS = 100;
    const N_TABLES = 10;
    const wdb = await AnyStore.create();
    const tables = [...Array(N_TABLES)].map((_, i) =>
      wdb.createTable(`table_${i}`, {
        name: "string",
        age: "i32",
        height: "f64",
      }),
    );

    function insertAndRemove() {
      for (const table of tables) {
        mockData.forEach((item, index) => {
          const key = AnyStore.i32(index);
          // table.insert(key, AnyStore.string(item.name), "name");
          const row = table.createRow(key);
          row.age = item.age;
          row.height = item.height;
        });
      }

      for (const table of tables) {
        mockData.forEach((_, index) => {
          const row = table.createRow(AnyStore.i32(index));
          row.delete();
        });
      }
    }

    for (let i = 0; i < N_REPETITIONS / 2; i++) {
      insertAndRemove();
    }
    const mem = wdb.memSize();
    for (let i = 0; i < N_REPETITIONS / 2; i++) {
      insertAndRemove();
    }
    //memory should not grow over time
    expect(wdb.memSize()).toBe(mem);
  });
});
