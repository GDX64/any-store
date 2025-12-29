async function main() {
  const importObj = {};

  const data = require("fs").readFileSync(
    "./target/wasm32-unknown-unknown/release/any_store.wasm"
  );
  const { instance } = await WebAssembly.instantiate(data, importObj);
  const vecPointer = instance.exports.create_vec();
  console.log("Vector stored at index:", vecPointer);
  instance.exports.push_vec(vecPointer, 4);
  console.log("Pushed 4 to vector at index:", vecPointer);
  const value = instance.exports.get_vec(vecPointer, 3);
  console.log("Value at index 3:", value);
}

main();
