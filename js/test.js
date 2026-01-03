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
  const { instance, module } = res;

  const vecPointer = instance.exports.create_vec();
  console.log("Vector stored at index:", vecPointer);
  instance.exports.push_vec(vecPointer, 4);
  console.log("Pushed 4 to vector at index:", vecPointer);
  const value = instance.exports.get_vec(vecPointer, 3);
  console.log("Value at index 3:", value);
  instance.exports.set_global_var(42);
  // const cloned = await WebAssembly.instantiate(data, importObj);
  // const globalValue = cloned.exports.get_global_var();
  // console.log("Global variable value:", globalValue);
}

main();
