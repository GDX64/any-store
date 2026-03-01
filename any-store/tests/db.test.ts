import { AnyStore } from "../src/WDB";
import { describe, expect, test, vi } from "vitest";
import fs from "fs";

vi.stubGlobal(
  "fetch",
  vi.fn(async (url: URL) => {
    const mod = fs.readFileSync(url.pathname.slice(1));
    return mod;
  }),
);

describe("Database Module", () => {
  test("should initialize the database correctly", async () => {
    const wdb = await AnyStore.create();
    const table = wdb.createTable("test_table", {
      name: "string",
      age: "i32",
      height: "f64",
      data: "blob",
    });
    const k1 = AnyStore.i32(123);
    const row = table.row(k1);
    row.name("Alice");
    row.age(30);
    row.height(1.75);
    row.data(new Uint8Array([1, 2, 3]));
    const row2 = table.row(AnyStore.i32(0));
    row2.name("Bob");

    const row1 = table.row(k1);

    //assert types
    const name1: string | null = row1.name();
    const age1: number | null = row1.age();
    const height1: number | null = row1.height();
    const data1: Uint8Array | null = row1.data();

    expect(name1).toBe("Alice");
    expect(age1).toBe(30);
    expect(height1).toBeCloseTo(1.75);
    expect(data1).toEqual(new Uint8Array([1, 2, 3]));
    row1.delete();

    expect(row1.name()).toBeNull();

    const row3 = table.row(AnyStore.i32(0));
    expect(row3.name()).toBe("Bob");

    row1.name(null);
    expect(row1.name()).toBeNull();
  });

  test("using accessors", async () => {
    const wdb = await AnyStore.create();
    const table = wdb.createTable("test_table", {
      name: "string",
      age: "i32",
    });
    const k1 = AnyStore.i32(123);
    const row = table.row(k1);
    row.name("Alice");
    row.age(30);

    expect(row.name()).toBe("Alice");
    expect(row.age()).toBe(30);
  });

  test("insert and remove random data", async () => {
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

    const N_REPETITIONS = 2;
    const N_TABLES = 5;
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
          const row = table.row(key);
          row.name(item.name);
          row.age(item.age);
          row.height(item.height);
        });

        mockData.forEach((item, index) => {
          const key = AnyStore.i32(index);
          const row = table.row(key);
          const rowData = row.getRow();
          expect(rowData[0]).toBe(item.name);
          expect(rowData[1]).toBe(item.age);
          expect(rowData[2]).toBeCloseTo(item.height);
          const name = row.name();
          const age = row.age();
          const height = row.height();
          expect(name).toBe(item.name);
          expect(age).toBe(item.age);
          expect(height).toBeCloseTo(item.height);

          row.delete();

          expect(row.name()).toBeNull();
          expect(row.age()).toBeNull();
          expect(row.height()).toBeNull();
        });
      }
    }

    for (let i = 0; i < N_REPETITIONS; i++) {
      insertAndRemove();
    }
  });

  test("add listener to row", async () => {
    const wdb = await AnyStore.create();
    const table = wdb.createTable("test_table", {
      counter: "i32",
    });
    const row = table.row(AnyStore.i32(1));
    const fn = vi.fn();
    const listenerID = row.addListener(fn);
    wdb.notifyAll();
    expect(fn).toHaveBeenCalledTimes(0);

    row.counter(0);

    wdb.notifyAll();
    wdb.notifyAll(); //even if we notify multiple times, the listener should be called only once
    expect(fn).toHaveBeenCalledTimes(1);

    row.removeListener(listenerID);

    row.counter(1);

    wdb.notifyAll();
    expect(fn).toHaveBeenCalledTimes(1);
  });

  test("cached row", async () => {
    const wdb = await AnyStore.create();
    const table = wdb.createTable("test_table", {
      counter: "i32",
    });
    const row = table.row(AnyStore.i32(1));

    const fn = vi.fn();

    row.cached(fn);
    expect(row.counter()).toBeNull();
    row.counter(0);

    expect(fn).toHaveBeenCalledTimes(0);

    wdb.notifyAll();
    expect(row.counter()).toBe(0);
    expect(fn).toHaveBeenCalledTimes(1);

    row.counter(1);

    expect(row.counter()).toBe(0); // because we are observing row, we need to wait until it is notified

    wdb.notifyAll();
    expect(fn).toHaveBeenCalledTimes(2);
    expect(row.counter()).toBe(1);
  });

  test("worker modules in the same thread", async () => {
    const wdb = await AnyStore.create();

    wdb.createTable("table1", { counter: "i32" }); //not used
    wdb.createTable("table2", { counter: "i32" }); //not used

    const table = wdb.createTable("hello", { counter: "i32" });
    const firstRow = table.row(AnyStore.i32(1));
    firstRow.counter(10);

    const module = wdb.createWorker();
    const other = await AnyStore.fromModule(module);

    const otherTable = other.getTable("hello", { counter: "i32" });
    if (!otherTable) {
      throw new Error("Table 'hello' not found in other module");
    }

    const otherRow = otherTable.row(AnyStore.i32(1));
    expect(otherRow.counter()).toBe(10);

    otherRow.counter(20);
    expect(firstRow.counter()).toBe(20);
  });

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
          const row = table.row(key);
          row.age(item.age);
          row.height(item.height);
        });
      }

      for (const table of tables) {
        mockData.forEach((_, index) => {
          const row = table.row(AnyStore.i32(index));
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
