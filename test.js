//const zeos = import('./pkg/zeos_orchard.js');
//zeos.then(async m => {
//    console.log(m.test1(null));
//    console.log(await m.run("mschoenebeck"));
//}).catch(console.error);
///////////////////////////////////////////////

const {test1, test_merkle_hash_fetch, test_merkle_path_fetch, test_get_table_rows, test_get_global, test_fetch_notes} = wasm_bindgen;

async function run_wasm() {
    // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`
    // `wasm_bindgen` was imported in `index.html`
    await wasm_bindgen('./pkg/zeos_orchard_bg.wasm');

    console.log('test.js loaded');

    // Run main WASM entry point
    // This will create a worker from within our Rust code compiled to WASM
    console.log(test_merkle_hash_fetch("0"));
    console.log(test_merkle_path_fetch("1", "6"));
    console.log(test_get_table_rows());
    console.log(test_get_global());
    console.log(test_fetch_notes());
}

run_wasm();
