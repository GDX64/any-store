import { vi } from "vitest";
import fs from "fs";

export function setupFetch() {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: URL) => {
      const mod = fs.readFileSync(url.pathname.slice(1));
      return mod;
    }),
  );
}
