import type { Table } from "./Table";
import type { ColMap, Something, ValueMap } from "./types";

export type Row<T extends ColMap> = {
  [K in keyof T as `${K & string}`]: ValueMap[T[K]] | null;
} & _Row<T>;

export class _Row<T extends ColMap> {
  private cache: Something["value"][] | null = null;

  constructor(
    private table: Table<T>,
    public rowID: number = 0,
    public readonly rowKey: Something,
  ) {}

  private load() {
    this.cache = this.table.getRowData(this.rowID);
  }

  cached(onUpdate?: () => void) {
    return this.addListener(() => {
      this.load();
      onUpdate?.();
    });
  }

  addListener(fn: () => void) {
    return this.table.addListenerToRow(this.rowID, fn);
  }

  delete() {
    return this.table.deleteRow(this.rowID);
  }

  removeListener(listenerID: number) {
    this.table.removeListenerFromRow(listenerID, this.rowID);
  }

  getRow(): Something["value"][] {
    if (this.cache) {
      return this.cache;
    }
    return this.table.getRowData(this.rowID);
  }
}
