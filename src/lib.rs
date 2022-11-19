//! # orchard
//!
//! ## Nomenclature
//!
//! All types in the `orchard` crate, unless otherwise specified, are Orchard-specific
//! types. For example, [`Address`] is documented as being a shielded payment address; we
//! implicitly mean it is an Orchard payment address (as opposed to e.g. a Sapling payment
//! address, which is also shielded).

#![cfg_attr(docsrs, feature(doc_cfg))]
// Temporary until we have more of the crate implemented.
#![allow(dead_code)]
// Catch documentation errors caused by code changes.
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]
//#![deny(unsafe_code)]
#![feature(async_fn_in_trait)]

mod action;
mod address;
pub mod builder;
pub mod bundle;
pub mod circuit;
pub mod contract;
mod constants;
pub mod keys;
pub mod note;
pub mod note_encryption;
pub mod primitives;
mod spec;
pub mod tree;
pub mod value;
pub mod zip32;

#[cfg(test)]
mod test_vectors;

pub use action::RawZAction;
pub use address::Address;
pub use bundle::Bundle;
extern crate nonempty;
use nonempty::{NonEmpty, nonempty};
pub use note::Note;
pub use tree::Anchor;

use crate::keys::SpendingKey;
use crate::keys::FullViewingKey;
use crate::contract::NoteEx;

use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
extern crate serde_json;

use rand::rngs::OsRng;

use crate::contract::*;

#[macro_use]
extern crate serde_derive;

const ENDPOINTS: NonEmpty<&'static str> = nonempty![
    "https://kylin-dsp-1.liquidapps.io"
];

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
pub struct ZEOSSpendingKey
{
    /// The actual spending key
    pub(crate) sk: SpendingKey
}

#[wasm_bindgen]
impl ZEOSSpendingKey
{
    /// create new spending key from seed phrase
    pub fn from_seed(seed: String) -> Self
    {
        ZEOSSpendingKey { sk: SpendingKey::from_zip32_seed(seed.as_bytes(), 0, 0).unwrap() }
    }
}

#[wasm_bindgen]
pub async fn test1(_js_objects: JsValue) -> String
{
    console_error_panic_hook::set_once();
    //let elements: Vec<JSSpendingKey> = serde_wasm_bindgen::from_value(js_objects).unwrap();
    
    let mut rng = OsRng.clone();
    NoteEx::from_parts(0, 0, Note::dummy(&mut rng, None, None).2).commitment()
}

#[wasm_bindgen]
pub async fn test_get_table_rows() -> JsValue
{
    // prepare POST request to fetch from EOSIO multiindex table
    let payload = EOSGetTableRowsPayload{
        code: "thezeostoken".to_string(),
        table: "mteosram".to_string(),
        scope: "thezeostoken".to_string(),
        index_position: "primary".to_string(),
        key_type: "uint64_t".to_string(),
        encode_type: "dec".to_string(),
        lower_bound: 0,
        upper_bound: 15,
        limit: 1,
        reverse: false,
        show_payer: false
    };
    
    let thezeostoken = TokenContract::new(ENDPOINTS.map(String::from));
    let res = thezeostoken.get_table_rows(&mut payload.clone()).await;
    JsValue::from_str(&serde_json::to_string(&res).unwrap())
}

#[wasm_bindgen]
pub async fn test_merkle_hash_fetch(index: String) -> JsValue
{
    let thezeostoken = TokenContract::new(ENDPOINTS.map(String::from));
    let mh = thezeostoken.get_merkle_hash(index.parse::<u64>().unwrap()).await;
    match mh {
        None => JsValue::NULL,
        Some(x) => JsValue::from_str(&hex::encode(x.1.to_bytes()))
    }
}

#[wasm_bindgen]
pub async fn test_merkle_path_fetch(leaf_index: String, leaf_count: String) -> JsValue
{
    // remember to set the correct merkle tree depth in constants.rs
    let mut thezeostoken = TokenContract::new(ENDPOINTS.map(String::from));
    let path = thezeostoken.get_merkle_path(leaf_index.parse::<u64>().unwrap(), leaf_count.parse::<u64>().unwrap()).await;

    let str = format!("{}, [({:?}), ({:?}), ({:?}), ({:?})]", path.position(), hex::encode(path.auth_path()[0].inner().0[0].to_le_bytes()), hex::encode(path.auth_path()[1].inner().0[0].to_le_bytes()), hex::encode(path.auth_path()[2].inner().0[0].to_le_bytes()), hex::encode(path.auth_path()[3].inner().0[0].to_le_bytes()));
    JsValue::from_str(&str)
}

#[wasm_bindgen]
pub async fn test_get_global() -> JsValue
{
    let thezeostoken = TokenContract::new(ENDPOINTS.map(String::from));
    let res = thezeostoken.get_global_state().await;
    JsValue::from_str(&serde_json::to_string(&res).unwrap())
}

#[wasm_bindgen]
pub async fn test_fetch_notes() -> JsValue
{   
    let thezeostoken = TokenContract::new(ENDPOINTS.map(String::from));
    let res = thezeostoken.get_encrypted_notes(0, 10).await;
    JsValue::from_str(&serde_json::to_string(&res).unwrap())
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}
