export type ColMap = Record<string, Something["tag"]>;

export type ValueMap = {
  i32: number;
  string: string;
  null: null;
  f64: number;
  blob: Uint8Array;
};

export type I32 = { tag: "i32"; value: number };
export type String = { tag: "string"; value: string };
export type Null = { tag: "null"; value: null };
export type F64 = { tag: "f64"; value: number };
export type Blob = { tag: "blob"; value: Uint8Array };

export type Something = I32 | String | Null | F64 | Blob;
