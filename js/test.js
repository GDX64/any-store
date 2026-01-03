async function main() {
  const memory = new WebAssembly.Memory({
    initial: 20,
    maximum: 40,
    shared: true,
  });
  const importObj = {
    env: {
      memory,
    },
  };

  const data = require("fs").readFileSync(
    "./target/wasm32-unknown-unknown/release/any_store.wasm"
  );

  const res = await WebAssembly.instantiate(data, importObj);
  const { instance } = res;
  //key
  instance.exports.something_push_i64_to_stack(43n);
  //value
  instance.exports.something_push_i64_to_stack(84n);
  const tableID = instance.exports.table_create();
  const col = 0;
  instance.exports.table_insert_from_stack(tableID, col);

  //key again
  instance.exports.something_push_i64_to_stack(43n);
  instance.exports.table_get_something(tableID, col);
  const result = instance.exports.something_pop_i64_from_stack();
  console.log("result:", result); // should be 84

  const { stringID } = createString(instance, memory, "hello world");

  //key
  instance.exports.something_push_i64_to_stack(123n);
  //value
  instance.exports.something_push_string(stringID);
  instance.exports.table_insert_from_stack(tableID, 1);

  //key again
  instance.exports.something_push_i64_to_stack(123n);
  instance.exports.table_get_something(tableID, 1);
  const resultID = instance.exports.something_pop_string_from_stack();
  const resultPointer = instance.exports.string_get_pointer(resultID);
  console.log("stringID:", resultID, resultPointer);
  const resultString = memory.buffer.slice(resultPointer, resultPointer + 11);
  const decodedString = new TextDecoder().decode(resultString);
  console.log("result string:", decodedString); // should be "hello world"
}

function createString(instance, memory, str) {
  const stringID = instance.exports.string_create(str.length);
  const stringPtr = instance.exports.string_get_pointer(stringID);
  const arr = new Uint8Array(memory.buffer);
  const helloBytes = new TextEncoder().encode(str);
  arr.set(helloBytes, stringPtr);
  return { stringID };
}

main();
