import { AnyStore } from "../src/AnyStore";
import { describe, expect, test } from "vitest";

describe("counter test", () => {
  test("counter", async () => {
    const db = await AnyStore.create();
    const table = db.createTable("counter", { value: "i32" });
    const row = table.createRow(AnyStore.i32(1));
    db.withLock(() => {
      row.value = 0;
      for (let i = 0; i < 1000_000; i++) {
        row.value += 1;
      }
    });
    expect(row.value).toBe(1000_000);
  });
});
