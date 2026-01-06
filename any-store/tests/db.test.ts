import fs from "fs";
import { WDB } from "../src/WDB";
import { describe, expect, test } from "vitest";
const wasmPath = "../target/wasm32-unknown-unknown/release/any_store.wasm";

describe("Database Module", () => {
  test("should initialize the database correctly", async () => {
    const data = fs.readFileSync(wasmPath);
    const wdb = await WDB.create(data);
    const table = wdb.createTable({
      name: "string",
      age: "i32",
      height: "f64",
    });
    const k1 = WDB.i32(123);
    table.insert(k1, WDB.string("Alice"), "name");
    table.insert(k1, WDB.i32(30), "age");
    table.insert(k1, WDB.f64(1.75), "height");
    table.insert(WDB.i32(0), WDB.string("Bob"), "name");
    wdb.commit();

    const row1 = table.row(k1);
    expect(row1.get("name")).toBe("Alice");
    expect(row1.get("age")).toBe(30);
    expect(row1.get("height")).toBeCloseTo(1.75);

    const row2 = table.row(WDB.i32(0));
    expect(row2.get("name")).toBe("Bob");
  });

  const mockData = Array.from({ length: 100 }, (_, i) => {
    return {
      age: Math.round(Math.random() * 100),
      height: Math.random() * 2,
      name: `Name_${i}`,
    };
  });

  test("insert random data", async () => {
    const TABLES = 10;
    const data = fs.readFileSync(wasmPath);
    const wdb = await WDB.create(data);
    for (let t = 0; t < TABLES; t++) {
      const table = wdb.createTable({
        name: "string",
        age: "i32",
        height: "f64",
      });

      mockData.forEach((item, index) => {
        const key = WDB.i32(index);
        table.insert(key, WDB.string(item.name), "name");
        table.insert(key, WDB.i32(item.age), "age");
        table.insert(key, WDB.f64(item.height), "height");
      });
      wdb.commit();
    }
  });
});
