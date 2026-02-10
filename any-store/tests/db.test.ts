import fs from "fs";
import { WDB } from "../src/WDB";
import { describe, expect, test } from "vitest";
const wasmPath = "../target/wasm32-unknown-unknown/release/any_store.wasm";

const data = fs.readFileSync(wasmPath);

describe("Database Module", () => {
  test("should initialize the database correctly", async () => {
    const wdb = await WDB.create(0, data);
    const table = wdb.createTable({
      name: "string",
      age: "i32",
      height: "f64",
      data: "blob",
    });
    const k1 = WDB.i32(123);
    table.insert(k1, WDB.string("Alice"), "name");
    table.insert(k1, WDB.i32(30), "age");
    table.insert(k1, WDB.f64(1.75), "height");
    table.insert(k1, WDB.blob(new Uint8Array([1, 2, 3]))!, "data");
    table.insert(WDB.i32(0), WDB.string("Bob"), "name");
    wdb.commit();

    const row1 = table.row(k1);
    expect(row1.get("name")).toBe("Alice");
    expect(row1.get("age")).toBe(30);
    expect(row1.get("height")).toBeCloseTo(1.75);
    expect(row1.get("data")).toEqual(new Uint8Array([1, 2, 3]));

    const row2 = table.row(WDB.i32(0));
    expect(row2.get("name")).toBe("Bob");
  });

  const mockData = new Map<
    number,
    { age: number; height: number; name: string }
  >();
  Array.from({ length: 100 }, (_, i) => {
    mockData.set(i, {
      age: Math.round(Math.random() * 100),
      height: Math.random() * 2,
      name: `Name_${i}`,
    });
  });

  test("insert random data", async () => {
    const TABLES = 2;
    const data = fs.readFileSync(wasmPath);
    const wdb = await WDB.create(0, data);
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

      mockData.forEach((item, index) => {
        const key = WDB.i32(index);
        const row = table.row(key);
        const rowData = row.getRow();
        expect(rowData[0]).toBe(item.name);
        expect(rowData[1]).toBe(item.age);
        expect(rowData[2]).toBeCloseTo(item.height);
        const name = row.get("name");
        const age = row.get("age");
        const height = row.get("height");
        expect(name).toBe(item.name);
        expect(age).toBe(item.age);
        expect(height).toBeCloseTo(item.height);
      });
    }
  });
});
