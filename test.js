//const zeos = import('./pkg/zeos_orchard.js');
//zeos.then(async m => {
//    console.log(m.test1(null));
//    console.log(await m.run("mschoenebeck"));
//}).catch(console.error);
///////////////////////////////////////////////

const {test1, test_merkle_hash_fetch, test_merkle_path_fetch, test_get_table_rows, test_get_global, test_fetch_notes, Wallet, test_proof_upload} = wasm_bindgen;

async function run_wasm() {
    // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`
    // `wasm_bindgen` was imported in `index.html`
    await wasm_bindgen('./pkg/zeos_orchard_bg.wasm');

    console.log('test.js loaded');

    // Run main WASM entry point
    // This will create a worker from within our Rust code compiled to WASM
    //console.log(test_merkle_hash_fetch("0"));
    //console.log(test_merkle_path_fetch("1", "6"));
    //console.log(test_get_table_rows());
    //console.log(test_get_global());
    //console.log(test_fetch_notes());
    //console.log(await test_proof_upload());

    /*
    let formData = new FormData();
    formData.append('strupload', '12345');
    await fetch("http://web3.zeos.one/uploadstr",
    {
        body: formData,
        method: 'post',
        mode: 'no-cors'
    })
    // 'no-cors' mode doesn't allow the browser to read any response content.
    // see: https://stackoverflow.com/a/54906434/2340535
    .then(response=>response.json())
    .then(data=>{ console.log(data); })
    */

    var sender = Wallet.new("This is the sender wallets seed string. It must be at least 32 characters long!")
    var receiver = Wallet.new("This is the receiver wallets seed string. It must be at least 32 characters long!")
    console.log(sender.to_json_string());
    
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
            d1: 10000,
            d2: 1397703940,
            sc: 6138663591592764928,
            memo: "This is a test!"
        }]
    }];
    
    var tx = JSON.parse(await sender.create_transaction(JSON.stringify(descs), JSON.stringify(auth)));
    console.log(JSON.stringify(tx, null, 2));
}

run_wasm();
