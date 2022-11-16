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

mod action;
mod address;
pub mod builder;
pub mod bundle;
pub mod circuit;
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
pub use note::Note;
pub use tree::Anchor;

use note::TransmittedNoteCiphertext;
use tree::{MerkleHashOrchard, MerklePath};
use pasta_curves::Fp;
use crate::note::{Nullifier, RandomSeed};
use crate::note_encryption::ENC_CIPHERTEXT_SIZE;
use crate::keys::SpendingKey;
use crate::tree::EMPTY_ROOTS;
use crate::value::NoteValue;
use crate::note::ExtractedNoteCommitment;
use crate::keys::FullViewingKey;

use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
extern crate serde_json;

use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use std::fmt;

use rand::rngs::OsRng;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use std::collections::HashMap;
use crate::constants::MERKLE_DEPTH_ORCHARD;

#[macro_use]
extern crate serde_derive;

#[wasm_bindgen]
#[derive(Debug)]
pub struct EOSTransmittedNoteCiphertext
{
    /// This notes global ID.
    id: String, //u64
    /// The current EOS block number when this note was added to the 
    /// global list of encrypted notes
    block_number: String, //u64
    /// The current leaf index of the merkle tree when this note was
    /// added to the global list of encrypted notes
    leaf_index: String, //u64
    /// The actual encrypted note
    encrypted_note: TransmittedNoteCiphertext
}

impl Serialize for TransmittedNoteCiphertext
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("TransmittedNoteCiphertext", 3)?;
        state.serialize_field("epk_bytes", &&hex::encode(self.epk_bytes))?;
        state.serialize_field("enc_ciphertext", &hex::encode(self.enc_ciphertext))?;
        state.serialize_field("out_ciphertext", &hex::encode(self.out_ciphertext))?;
        state.end()
    }
}

#[wasm_bindgen]
impl EOSTransmittedNoteCiphertext
{
    pub fn from_parts(
        id: String,
        block_number: String,
        leaf_index: String,
        epk_bytes_str: String,
        enc_ciphertext_str: String,
        out_ciphertext_str: String
    ) -> Self
    {
        assert!(epk_bytes_str.len() == 32 * 2);
        let mut epk_bytes = [0; 32];
        hex::decode_to_slice(epk_bytes_str, &mut epk_bytes).expect("Decoding of 'epk_bytes_str' failed");
        
        assert!(enc_ciphertext_str.len() == ENC_CIPHERTEXT_SIZE * 2);
        let mut enc_ciphertext = [0; ENC_CIPHERTEXT_SIZE];
        hex::decode_to_slice(enc_ciphertext_str, &mut enc_ciphertext).expect("Decoding of 'enc_ciphertext_str' failed");

        assert!(out_ciphertext_str.len() == 80 * 2);
        let mut out_ciphertext = [0; 80];
        hex::decode_to_slice(out_ciphertext_str, &mut out_ciphertext).expect("Decoding of 'out_ciphertext_str' failed");

        EOSTransmittedNoteCiphertext{
            id,
            block_number,
            leaf_index,
            encrypted_note: TransmittedNoteCiphertext { epk_bytes, enc_ciphertext, out_ciphertext }
        }
    }

    // try decrypt as receiver
    // TODO

    // try decrypt as sender
    // TODO

}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EOSNote
{
    /// The current EOS block number when this note was added to the 
    /// global list of encrypted notes
    pub(crate) block_number: String, //u64
    /// The current leaf index of the merkle tree when this note was
    /// added to the global list of encrypted notes
    pub(crate) leaf_index: String, //u64
    /// The actual Note
    pub(crate) note: Note
}

impl Serialize for Note
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 9 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Note", 9)?;
        state.serialize_field("header", &self.header().to_string())?;
        state.serialize_field("recipient", &hex::encode(self.recipient().to_raw_address_bytes()))?;
        state.serialize_field("d1", &self.d1().inner().to_string())?;
        state.serialize_field("d2", &self.d2().inner().to_string())?;
        state.serialize_field("sc", &self.sc().inner().to_string())?;
        state.serialize_field("nft", &self.nft().inner())?;
        state.serialize_field("rho", &self.rho().to_bytes())?;
        state.serialize_field("rseed", self.rseed().as_bytes())?;
        state.serialize_field("memo", &hex::encode(self.memo()))?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Note
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field { Header, Recipient, D1, D2, Sc, Nft, Rho, Rseed, Memo }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`header` or `recipient` or `d1` or `d2` or `sc` or `nft` or `rho` or `rseed` or `memo`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "header" => Ok(Field::Header),
                            "recipient" => Ok(Field::Recipient),
                            "d1" => Ok(Field::D1),
                            "d2" => Ok(Field::D2),
                            "sc" => Ok(Field::Sc),
                            "nft" => Ok(Field::Nft),
                            "rho" => Ok(Field::Rho),
                            "rseed" => Ok(Field::Rseed),
                            "memo" => Ok(Field::Memo),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct NoteVisitor;

        impl<'de> Visitor<'de> for NoteVisitor {
            type Value = Note;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Note")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Note, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let header: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let header = header.parse::<u64>().unwrap();
                let recipient_str: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let mut recipient = [0; 43];
                assert_eq!(hex::decode_to_slice(recipient_str, &mut recipient), Ok(()));
                let recipient = Address::from_raw_address_bytes(&recipient).unwrap();
                let d1: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let d1 = d1.parse::<u64>().unwrap();
                let d1 = NoteValue::from_raw(d1);
                let d2: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let d2 = d2.parse::<u64>().unwrap();
                let d2 = NoteValue::from_raw(d2);
                let sc: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let sc = sc.parse::<u64>().unwrap();
                let sc = NoteValue::from_raw(sc);
                let nft = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;
                let nft = NoteValue::from_raw(nft);
                let rho = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?;
                let rho = Nullifier::from_bytes(&rho).unwrap();
                let rseed = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?;
                let rseed = RandomSeed::from_bytes(rseed, &rho).unwrap();
                let memo_str: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(8, &self))?;
                let mut memo = [0; 512];
                assert_eq!(hex::decode_to_slice(memo_str, &mut memo), Ok(()));
                Ok(Note::from_parts(header, recipient, d1, d2, sc, nft, rho, rseed, memo))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Note, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut header = None;
                let mut recipient_str = None;
                let mut d1 = None;
                let mut d2 = None;
                let mut sc = None;
                let mut nft = None;
                let mut rho = None;
                let mut rseed = None;
                let mut memo_str = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Header => {
                            if header.is_some() {
                                return Err(de::Error::duplicate_field("header"));
                            }
                            header = Some(map.next_value()?);
                        }
                        Field::Recipient => {
                            if recipient_str.is_some() {
                                return Err(de::Error::duplicate_field("recipient"));
                            }
                            recipient_str = Some(map.next_value()?);
                        }
                        Field::D1 => {
                            if d1.is_some() {
                                return Err(de::Error::duplicate_field("d1"));
                            }
                            d1 = Some(map.next_value()?);
                        }
                        Field::D2 => {
                            if d2.is_some() {
                                return Err(de::Error::duplicate_field("d2"));
                            }
                            d2 = Some(map.next_value()?);
                        }
                        Field::Sc => {
                            if sc.is_some() {
                                return Err(de::Error::duplicate_field("sc"));
                            }
                            sc = Some(map.next_value()?);
                        }
                        Field::Nft => {
                            if nft.is_some() {
                                return Err(de::Error::duplicate_field("nft"));
                            }
                            nft = Some(map.next_value()?);
                        }
                        Field::Rho => {
                            if rho.is_some() {
                                return Err(de::Error::duplicate_field("rho"));
                            }
                            rho = Some(map.next_value()?);
                        }
                        Field::Rseed => {
                            if rseed.is_some() {
                                return Err(de::Error::duplicate_field("rseed"));
                            }
                            rseed = Some(map.next_value()?);
                        }
                        Field::Memo => {
                            if memo_str.is_some() {
                                return Err(de::Error::duplicate_field("memo"));
                            }
                            memo_str = Some(map.next_value()?);
                        }
                    }
                }
                let header: String = header.ok_or_else(|| de::Error::missing_field("header"))?;
                let header = header.parse::<u64>().unwrap();
                let recipient_str: String = recipient_str.ok_or_else(|| de::Error::missing_field("recipient"))?;
                let mut recipient = [0; 43];
                assert_eq!(hex::decode_to_slice(recipient_str, &mut recipient), Ok(()));
                let recipient = Address::from_raw_address_bytes(&recipient).unwrap();
                let d1: String = d1.ok_or_else(|| de::Error::missing_field("d1"))?;
                let d1 = d1.parse::<u64>().unwrap();
                let d1 = NoteValue::from_raw(d1);
                let d2: String = d2.ok_or_else(|| de::Error::missing_field("d2"))?;
                let d2 = d2.parse::<u64>().unwrap();
                let d2 = NoteValue::from_raw(d2);
                let sc: String = sc.ok_or_else(|| de::Error::missing_field("sc"))?;
                let sc = sc.parse::<u64>().unwrap();
                let sc = NoteValue::from_raw(sc);
                let nft: u64 = nft.ok_or_else(|| de::Error::missing_field("nft"))?;
                let nft = NoteValue::from_raw(nft);
                let rho: [u8; 32] = rho.ok_or_else(|| de::Error::missing_field("rho"))?;
                let rho = Nullifier::from_bytes(&rho).unwrap();
                let rseed: [u8; 32] = rseed.ok_or_else(|| de::Error::missing_field("rseed"))?;
                let rseed = RandomSeed::from_bytes(rseed, &rho).unwrap();
                let memo_str: String = memo_str.ok_or_else(|| de::Error::missing_field("memo"))?;
                let mut memo = [0; 512];
                assert_eq!(hex::decode_to_slice(memo_str, &mut memo), Ok(()));

                Ok(Note::from_parts(header, recipient, d1, d2, sc, nft, rho, rseed, memo))
            }
        }

        const FIELDS: &'static [&'static str] = &["header", "recipient", "d1", "d2", "sc", "nft", "rho", "rseed", "memo"];
        deserializer.deserialize_struct("Note", FIELDS, NoteVisitor)
    }
}

#[wasm_bindgen]
impl EOSNote
{
    /// create a new note from JS obj with wasm bindings
    pub fn from(obj: JsValue) -> Self
    {
        console_error_panic_hook::set_once();
        let note: Self = serde_wasm_bindgen::from_value(obj).unwrap();
        note
    }

    pub fn commitment(&self) -> String
    {
        let cm: ExtractedNoteCommitment = self.note.commitment().into();
        let mut res = [0; 32];
        res[0..8].copy_from_slice(&cm.inner().0[0].to_le_bytes());
        res[8..16].copy_from_slice(&cm.inner().0[1].to_le_bytes());
        res[16..24].copy_from_slice(&cm.inner().0[2].to_le_bytes());
        res[24..32].copy_from_slice(&cm.inner().0[3].to_le_bytes());
        hex::encode(res)
    }

    pub fn nullifier(&self, js_sk: JsValue) -> String
    {
        let sk: ZEOSSpendingKey = serde_wasm_bindgen::from_value(js_sk).unwrap();
        let nf = self.note.nullifier(&FullViewingKey::from(&sk.sk));
        let mut res = [0; 32];
        res[0..8].copy_from_slice(&nf.inner().0[0].to_le_bytes());
        res[8..16].copy_from_slice(&nf.inner().0[1].to_le_bytes());
        res[16..24].copy_from_slice(&nf.inner().0[2].to_le_bytes());
        res[24..32].copy_from_slice(&nf.inner().0[3].to_le_bytes());
        hex::encode(res)
    }
}

impl EOSNote
{
    pub fn from_parts(bn: u64, li: u64, note: Note) -> Self
    {
        EOSNote { block_number: bn.to_string(), leaf_index: li.to_string(), note }
    }
}

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZActionDesc
{
    /// ...
    pub(crate) za_type: u64,
    pub(crate) to: String, // EOS account for BURN actions or shielded address otherwise
    pub(crate) d1: u64,
    pub(crate) d2: u64,
    pub(crate) sc: u64,
    pub(crate) memo: String,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EOSAuthorization
{
    pub(crate) actor: String,
    pub(crate) permission: String,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EOSAction
{
    pub(crate) account: String,
    pub(crate) name: String,
    pub(crate) authorization: Vec<EOSAuthorization>,
    /// JSON string (unpacked EOSAction) or HEX string (packed EOSAction)
    pub(crate) data: String,
}

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
pub struct EOSActionDesc
{
    /// ...
    pub(crate) action: EOSAction,

    /// If this action contains zactions it MUST be wrapped in a step() call. In this case
    /// all ZActionDescs in this list are parsed, processed and their corresponding ZActions
    /// are serialized and added to the front of the serialized 'data' String of this EOSAction.
    pub(crate) zaction_descs: Vec<ZActionDesc>
}


#[wasm_bindgen]
pub async fn test1(_js_objects: JsValue) -> String
{
    console_error_panic_hook::set_once();
    //let elements: Vec<JSSpendingKey> = serde_wasm_bindgen::from_value(js_objects).unwrap();
    
    let mut rng = OsRng.clone();
    EOSNote::from_parts(0, 0, Note::dummy(&mut rng, None, None).2).commitment()
}

pub async fn fetch_merkle_hash(index: u64) -> (u64, MerkleHashOrchard)
{
    // prepare POST request to fetch from EOSIO multiindex table
    let body = format!("{{ \"code\": \"thezeostoken\", \"table\": \"mteosram\", \"scope\": \"thezeostoken\", \"index_position\": \"primary\", \"key_type\": \"uint64_t\", \"lower_bound\": \"{}\", \"upper_bound\": \"{}\" }}", index.to_string(), index.to_string());
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&JsValue::from_str(&body)));
    let url = "https://kylin-dsp-1.liquidapps.io/v1/chain/get_table_rows";
    let request = Request::new_with_str_and_init(&url, &opts).unwrap();
    request
        .headers()
        .set("Accept", "application/json").unwrap();
    
    // send http request using browser window's fetch
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
    
    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();
    let str = resp.text()
        .map(JsFuture::from).unwrap()
        .await.unwrap()
        .as_string()
        .expect("fetch: Response expected `String` after .text()");
    
    // str has the following format:
    // "{\"rows\":[\"0f00000000000000f9ffffff84a9c3cf3e30e5be1bd11110ffffffffffffffffffffffffffffff3f\"],\"more\":false,\"next_key\":\"\"}"
    // extract serialized node data and parse to (index, MerkleHash) tuple
    let str: String = str.chars().skip(10).take(40 * 2).collect();
    let mut arr = [0; 40];
    assert!(hex::decode_to_slice(str, &mut arr).is_ok());
    let index = u64::from_le_bytes(arr[0..8].try_into().unwrap());
    let value = MerkleHashOrchard::from(Fp([
        u64::from_le_bytes(arr[ 8..16].try_into().unwrap()),
        u64::from_le_bytes(arr[16..24].try_into().unwrap()),
        u64::from_le_bytes(arr[24..32].try_into().unwrap()),
        u64::from_le_bytes(arr[32..40].try_into().unwrap())
    ]));
    
    (index, value)
}

macro_rules! MT_ARR_LEAF_ROW_OFFSET     { ($d:expr) => { (1 << ($d)) - 1 }; }
macro_rules! MT_ARR_FULL_TREE_OFFSET    { ($d:expr) => { (1 << (($d) + 1)) - 1 }; }
macro_rules! MT_NUM_LEAVES              { ($d:expr) => { 1 << ($d) }; }
pub async fn get_merkle_path(
    leaf_index: u64,
    leaf_count: u64,
    node_buffer: &mut HashMap<u64, MerkleHashOrchard>
) -> MerklePath
{
    // only merkle trees with depth up to 32 are supported by the circuit design
    assert!(MERKLE_DEPTH_ORCHARD <= 32);
    // initialize return values: position is the leaf_index in the >local< tree
    let position = (leaf_index % MT_NUM_LEAVES!(MERKLE_DEPTH_ORCHARD)) as u32;
    let mut auth_path = vec![EMPTY_ROOTS[0]; MERKLE_DEPTH_ORCHARD];

    // calculate tree offset
    let tree_idx = leaf_index / MT_ARR_FULL_TREE_OFFSET!(MERKLE_DEPTH_ORCHARD);
    let tos = tree_idx * MT_ARR_FULL_TREE_OFFSET!(MERKLE_DEPTH_ORCHARD);
    let mut idx = MT_ARR_LEAF_ROW_OFFSET!(MERKLE_DEPTH_ORCHARD) + position as u64;
    let mut last_node_in_row = MT_ARR_LEAF_ROW_OFFSET!(MERKLE_DEPTH_ORCHARD) + leaf_count % MT_NUM_LEAVES!(MERKLE_DEPTH_ORCHARD) - 1;

    // walk through the tree (bottom to root)
    for d in 0..MERKLE_DEPTH_ORCHARD
    {
        //let log_str = format!("d: {}, tree_idx: {}, tos: {}, idx: {}, last_node_in_row: {}", d, tree_idx, tos, idx, last_node_in_row);log(&log_str);
        // if array index of node is uneven it is always the left child
        let is_left_child = 1 == idx % 2;
        // determine sister node
        let sis_idx = if is_left_child { idx + 1 } else { idx - 1 };
        // add sister node to auth_path
        let sis_idx_tos = tos + sis_idx;
        auth_path[d] = if node_buffer.contains_key(&sis_idx_tos) {
            node_buffer[&sis_idx_tos]
        } else {
            // if the sister index is greater than last_node_in_row it is an empty root
            let (i, v) = if sis_idx > last_node_in_row { 
                (sis_idx_tos, EMPTY_ROOTS[d]) 
            } else {
                fetch_merkle_hash(sis_idx_tos).await
            };
            node_buffer.insert(i, v);
            v
        };
        // set idx and last_node_in_row to parent node indices:
        // left child's array index divided by two (integer division) equals array index of parent node
        idx = if is_left_child { idx / 2 } else { sis_idx / 2 };
        last_node_in_row = if 1 == last_node_in_row % 2 { last_node_in_row / 2 } else { (last_node_in_row-1) / 2 };
    }

    assert_eq!(auth_path.len(), MERKLE_DEPTH_ORCHARD);
    MerklePath::from_parts(position, auth_path.try_into().unwrap())
}

#[wasm_bindgen]
pub async fn test_merkle_hash_fetch(index: String) -> JsValue
{
    let mh = fetch_merkle_hash(index.parse::<u64>().unwrap()).await;
    JsValue::from_str(&hex::encode(mh.1.to_bytes()))
}

#[wasm_bindgen]
pub async fn test_merkle_path_fetch(leaf_index: String) -> JsValue
{
    let mut nb: HashMap<u64, MerkleHashOrchard> = HashMap::new();
    let path = get_merkle_path(leaf_index.parse::<u64>().unwrap(), 10, &mut nb).await;
    let path = get_merkle_path(leaf_index.parse::<u64>().unwrap()+1, 10, &mut nb).await;

    let str = format!("{}, [({:?}), ({:?}), ({:?}), ({:?})]", path.position(), hex::encode(path.auth_path()[0].inner().0[0].to_le_bytes()), hex::encode(path.auth_path()[1].inner().0[0].to_le_bytes()), hex::encode(path.auth_path()[2].inner().0[0].to_le_bytes()), hex::encode(path.auth_path()[3].inner().0[0].to_le_bytes()));
    JsValue::from_str(&str)
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