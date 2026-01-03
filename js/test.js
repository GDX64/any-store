async function main() {
  const importObj = {
    env: {
      memory: new WebAssembly.Memory({
        initial: 20,
        maximum: 40,
        shared: true,
      }),
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
  const tablePtr = instance.exports.table_create();
  const col = 0;
  instance.exports.table_insert_from_stack(tablePtr, col);

  //key again
  instance.exports.something_push_i64_to_stack(43n);
  instance.exports.table_get_something(tablePtr, col);
  const result = instance.exports.something_pop_i64_from_stack();
  console.log("result:", result); // should be 84
}

main();
