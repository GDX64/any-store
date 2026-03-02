import type { AnyStore } from "./AnyStore";
import { _Row, type Row } from "./Row";
import type { ColMap, Something, ValueMap } from "./types";

export class Table<T extends ColMap> {
  colMap: Map<string, number> = new Map();
  private rowConstructor: typeof _Row;
  constructor(
    private tags: T,
    private tableID: number,
    private wdb: AnyStore,
  ) {
    Object.keys(tags).forEach((colName, index) => {
      this.colMap.set(colName, index);
    });

    class ThisRow extends _Row<any> {}

    for (const col in this.tags) {
      const colIndex = this.colMap.get(col)!;
      const tag = this.tags[col];
      const set: any = new Function(
        "value",
        `
        this.table._insert(this.rowID, value, "${col}", "${tag}");
        return value;
        `,
      );

      const get: any = new Function(`
        if(!arguments.length) {
          if(this.cache) {
            return this.cache[${colIndex}];
          }
          return this.table.wdb.getFromTable(this.table.tableID, this.rowID, ${colIndex});
        }`);

      Object.defineProperties(ThisRow.prototype as any, {
        [col]: {
          configurable: true,
          get: get,
          set: set,
          enumerable: true,
        },
      });
    }

    this.rowConstructor = ThisRow;
  }

  clear() {
    this.wdb.clearTable(this.tableID);
  }

  private tagOf(colName: keyof T): Something["tag"] {
    return this.tags[colName];
  }

  addListenerToRow(rowID: number, fn: () => void) {
    return this.wdb.addListenerToRow(this.tableID, rowID, fn);
  }

  getRowData(rowID: number): Something["value"][] {
    return this.wdb.getRowFromTable(this.tableID, rowID);
  }

  _insert(rowID: number, value: unknown, colName: keyof T) {
    const col = this.colMap.get(colName as string);
    this.wdb.insertOnTable(
      this.tableID,
      col!,
      rowID,
      value,
      this.tagOf(colName),
    );
  }

  removeListenerFromRow(listenerID: number, rowID: number) {
    this.wdb.removeListenerFromRow(this.tableID, rowID, listenerID);
  }

  get(rowID: number, colName: keyof T): Something["value"] | null {
    const col = this.colMap.get(colName as string);
    return this.wdb.getFromTable(this.tableID, rowID, col!);
  }

  deleteRow(rowID: number) {
    this.wdb.deleteRowFromTable(this.tableID, rowID);
  }

  createRow(key: Something) {
    const id = this.wdb.createRow(key, this.tableID);
    return new this.rowConstructor<T>(this, id, key) as Row<T>;
  }

  getRow(key: Something) {
    const rowID = this.wdb.getRowID(this.tableID, key);
    if (rowID === null) {
      return null;
    }
    return new this.rowConstructor<T>(this, rowID, key) as Row<T>;
  }

  where<K extends keyof T>(colName: K, value: ValueMap[T[K]]): number[] {
    return this.wdb.withColsEqual(
      this.tableID,
      this.colMap.get(colName as string)!,
      value,
      this.tagOf(colName),
    );
  }
}
