// Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init, { initThreadPool, Wallet } from './pkg/zeos_orchard.js';
//const {initThreadPool, test1, test_merkle_hash_fetch, test_merkle_path_fetch, test_get_table_rows, test_get_global, test_fetch_notes, Wallet, test_proof_upload} = wasm_bindgen;
console.log(`Worker created`);
//async function run_wasm() {
    // First up we need to actually load the wasm file, so we use the
    // default export to inform it where the wasm file is located on the
    // server, and then we wait on the returned promise to wait for the
    // wasm to be loaded.
    //
    // It may look like this: `await init('./pkg/without_a_bundler_bg.wasm');`,
    // but there is also a handy default inside `init` function, which uses
    // `import.meta` to locate the wasm file relatively to js file.
    //
    // Note that instead of a string you can also pass in any of the
    // following things:
    //
    // * `WebAssembly.Module`
    //
    // * `ArrayBuffer`
    //
    // * `Response`
    //
    // * `Promise` which returns any of the above, e.g. `fetch("./path/to/wasm")`
    //
    // This gives you complete control over how the module is loaded
    // and compiled.
    //
    // Also note that the promise, when resolved, yields the wasm module's
    // exports which is the same as importing the `*_bg` module in other
    // modes
    await init();
    //await wasm_bindgen('./pkg/zeos_orchard_bg.wasm');
    console.log('wasm loaded');
    console.log(navigator.hardwareConcurrency);

    // Thread pool initialization with the given number of threads
    // (pass `navigator.hardwareConcurrency` if you want to use all cores).
    await initThreadPool(8);

    // And afterwards we can use all the functionality defined in wasm.
    console.log("create wallets...");
    var sender = Wallet.new("This is the sender wallets seed string. It must be at least 32 characters long!")
    //var receiver = Wallet.new("This is the receiver wallets seed string. It must be at least 32 characters long!")
    console.log("sender wallet: " + sender.to_string());

    var auth = [{actor: "newstock1dex", permission: "active"}];
    var descs = [{
        action: {
            account: "eosio.token",
            name: "transfer",
            authorization: auth,
            data: "{\"from\":\"newstock1dex\", \"to\":\"thezeostoken\", \"quantity\":\"1.0000 EOS\", \"memo\":\"miau\"}"
        },
        zaction_descs: []
    }, {
        action: {
            account: "thezeostoken",
            name: "exec",
            authorization: [{actor: "thezeostoken", permission: "active"}],
            data: ""
        },
        zaction_descs: [{
            za_type: 1, //ZA_MINTFT,
            to: sender.address(0),
            d1: "10000",
            d2: "1397703940",
            sc: "thezeostoken",
            memo: "This is a test!"
        }]
    }];

    var tx = JSON.parse(await sender.create_transaction(JSON.stringify(descs), JSON.stringify(auth)));
    console.log(JSON.stringify(tx, null, 2));

//}
//run_wasm();
