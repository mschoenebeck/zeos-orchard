//! Structs related to the EOS smart contract of the ZEOS Orchard application.

use crate::tree::{MerkleHashOrchard, MerklePath};
use nonempty::NonEmpty;
use pasta_curves::Fp;
use crate::note::{Note, TransmittedNoteCiphertext, Nullifier, RandomSeed, ExtractedNoteCommitment};
use crate::note_encryption::{ENC_CIPHERTEXT_SIZE, try_note_decryption, try_output_recovery_with_ovk};
use crate::note_encryption::OUT_CIPHERTEXT_SIZE;
use crate::tree::EMPTY_ROOTS;
use crate::value::NoteValue;
use crate::builder::HasMerkleTree;
use crate::keys::PreparedIncomingViewingKey;
use crate::keys::OutgoingViewingKey;
use crate::address::Address;
use crate::constants::MERKLE_DEPTH_ORCHARD;
use crate::eosio::value_to_name;
extern crate console_error_panic_hook;
extern crate serde_json;
use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use std::fmt;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, FormData};


// helper macros for merkle tree operations
macro_rules! MT_ARR_LEAF_ROW_OFFSET     { ($d:expr) => { (1 << ($d)) - 1 }; }
macro_rules! MT_ARR_FULL_TREE_OFFSET    { ($d:expr) => { (1 << (($d) + 1)) - 1 }; }
macro_rules! MT_NUM_LEAVES              { ($d:expr) => { 1 << ($d) }; }

#[derive(Debug, Serialize)]
pub struct TransmittedNoteCiphertextEx
{
    /// This notes global ID.
    id: u64,
    /// The current EOS block number when this note was added to the 
    /// global list of encrypted notes
    block_number: u64,
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

impl TransmittedNoteCiphertextEx
{
    /// Try to decrypt note as receiver
    pub fn try_decrypt_as_receiver(
        &self,
        ivk: &PreparedIncomingViewingKey
    ) -> Option<NoteEx>
    {
        match try_note_decryption(ivk, &self.encrypted_note)
        {
            Some(decrypted_note) => {
                Some(NoteEx {
                    id: self.id,
                    block_number: self.block_number,
                    note: decrypted_note
                })
            },
            None => None,
        }
    }

    /// Try to decrypt note as sender
    pub fn try_decrypt_as_sender(
        &self,
        ovk: &OutgoingViewingKey
    ) -> Option<NoteEx>
    {
        match try_output_recovery_with_ovk(ovk, &self.encrypted_note)
        {
            Some(decrypted_note) => {
                Some(NoteEx {
                    id: self.id,
                    block_number: self.block_number,
                    note: decrypted_note
                })
            },
            None => None,
        }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteEx
{
    // The global id of this note
    pub(crate) id: u64,
    /// The current EOS block number when this note was added to the 
    /// global list of encrypted notes
    pub(crate) block_number: u64,
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
                Ok(Note::from_parts(header, recipient, d1, d2, sc, nft, rho, rseed, memo).unwrap())
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

                Ok(Note::from_parts(header, recipient, d1, d2, sc, nft, rho, rseed, memo).unwrap())
            }
        }

        const FIELDS: &'static [&'static str] = &["header", "recipient", "d1", "d2", "sc", "nft", "rho", "rseed", "memo"];
        deserializer.deserialize_struct("Note", FIELDS, NoteVisitor)
    }
}

impl NoteEx
{
/*
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
*/
}

/// Payload struct to fetch table rows of an EOSIO smart contract.
/// See also: https://developers.eos.io/manuals/eos/v2.2/nodeos/plugins/chain_api_plugin/api-reference/index#operation/get_table_rows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EOSGetTableRowsPayload
{
    pub code: String,               // required
    pub table: String,              // required
    pub scope: String,              // required
    pub index_position: String,     // 'primary', 'secondary', 'tertiary', 'fourth', 'fifth', 'sixth', 'seventh', 'eighth', 'ninth' or 'tenth', default: 'primary'?
    pub key_type: String,           // 'uint64_t' or 'name', default: 'uint64_t'?
    pub encode_type: String,        // 'dec' or 'hex', default: 'dec'
    pub lower_bound: String,
    pub upper_bound: String,
    pub limit: i32,                 // default: 10
    pub reverse: bool,              // default: false
    pub show_payer: bool            // default: false
}

/// Response struct to fetch table rows of an EOSIO smart contract.
/// See also: https://developers.eos.io/manuals/eos/v2.2/nodeos/plugins/chain_api_plugin/api-reference/index#operation/get_table_rows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EOSGetTableRowsResponse
{
    pub rows: Vec<String>,
    pub more: bool,
    pub next_key: String
}

// Required to de-/serialize u64 <-> String for use in JSON strings
// From: https://github.com/serde-rs/json/issues/329#issuecomment-305608405
mod string
{
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Serializer, Deserialize, Deserializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where T: Display,
              S: Serializer
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
        where T: FromStr,
              T::Err: Display,
              D: Deserializer<'de>
    {
        let str = String::deserialize(deserializer)?;
        if str.is_empty()
        {
            // if field is empty string assume zero
            "0".parse().map_err(de::Error::custom)
        }
        else
        {
            str.parse().map_err(de::Error::custom)
        }
    }
}

/// Represents singleton table 'global' of the ZEOS token contract
/// See also: thezeostoken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Global
{
    #[serde(with = "string")]
    pub note_count: u64,
    #[serde(with = "string")]
    pub leaf_count: u64,
    #[serde(with = "string")]
    pub tree_depth: u64,
}

/// Represents the ZEOS token contract
/// See also: thezeostoken
#[derive(Debug)]
pub struct TokenContract
{
    endpoints: NonEmpty<String>,
    node_buffer: HashMap<u64, MerkleHashOrchard>
}

impl HasMerkleTree for TokenContract
{
    async fn get_sister_path(
        &mut self,
        array_index: u64,
        leaf_count: u64,
    ) -> MerklePath
    {
        // only merkle trees with depth up to 32 are supported by the circuit design
        assert!(MERKLE_DEPTH_ORCHARD <= 32);
        let tree_index = array_index / MT_ARR_FULL_TREE_OFFSET!(MERKLE_DEPTH_ORCHARD);
        let latest_tree_index = leaf_count / MT_NUM_LEAVES!(MERKLE_DEPTH_ORCHARD);
        let mut last_node_in_row = {
            if tree_index == latest_tree_index
            {
                MT_ARR_LEAF_ROW_OFFSET!(MERKLE_DEPTH_ORCHARD) + leaf_count % MT_NUM_LEAVES!(MERKLE_DEPTH_ORCHARD) - 1
            }
            else
            {
                // last node/leaf in tree
                MT_ARR_LEAF_ROW_OFFSET!(MERKLE_DEPTH_ORCHARD) + MT_NUM_LEAVES!(MERKLE_DEPTH_ORCHARD) - 1
            }
        };
        let mut idx = array_index % MT_ARR_FULL_TREE_OFFSET!(MERKLE_DEPTH_ORCHARD);
        // check if index is really a leaf (and not a merkle node somewhere in the middle of the tree)
        assert!(idx >= MT_ARR_LEAF_ROW_OFFSET!(MERKLE_DEPTH_ORCHARD));
        // initialize return values: position is the leaf_index in the >local< tree
        let position = (idx - MT_ARR_LEAF_ROW_OFFSET!(MERKLE_DEPTH_ORCHARD)) as u32;
        let mut auth_path = vec![EMPTY_ROOTS[0]; MERKLE_DEPTH_ORCHARD];
        // calculate tree offset
        let tos = tree_index * MT_ARR_FULL_TREE_OFFSET!(MERKLE_DEPTH_ORCHARD);

        // walk through the tree (bottom to root)
        for d in 0..MERKLE_DEPTH_ORCHARD
        {
            // if array index of node is uneven it is always the left child
            let is_left_child = 1 == idx % 2;
            // determine sister node
            let sis_idx = if is_left_child { idx + 1 } else { idx - 1 };
            // add sister node to auth_path
            let sis_idx_tos = tos + sis_idx;
            auth_path[d] = if self.node_buffer.contains_key(&sis_idx_tos) {
                self.node_buffer[&sis_idx_tos]
            } else {
                // if the sister index is greater than last_node_in_row it is an empty root
                let v = if sis_idx > last_node_in_row {
                    EMPTY_ROOTS[d]
                } else {
                    self.get_merkle_hash(sis_idx_tos).await.unwrap()
                };
                self.node_buffer.insert(sis_idx_tos, v);
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

    async fn get_merkle_index(
        &self,
        hash: ExtractedNoteCommitment
    ) -> Option<u64>
    {
        let hash_str = format!("{}{}{}{}",
            hex::encode(hash.inner().0[0].to_le_bytes()),
            hex::encode(hash.inner().0[1].to_le_bytes()),
            hex::encode(hash.inner().0[2].to_le_bytes()),
            hex::encode(hash.inner().0[3].to_le_bytes())
        );
        // prepare POST request to fetch from EOSIO multiindex table
        let payload = EOSGetTableRowsPayload{
            code: "thezeostoken".to_string(),
            table: "mteosram".to_string(),
            scope: "thezeostoken".to_string(),
            index_position: "secondary".to_string(),
            key_type: "sha256".to_string(),
            encode_type: "hex".to_string(),
            lower_bound: hash_str.clone(),
            upper_bound: hash_str,
            limit: 1,
            reverse: false,
            show_payer: false
        };

        let res = self.get_table_rows(&mut payload.clone()).await;
        if res.rows.len() == 0
        {
            return None;
        }
        // extract serialized node data and parse 'index' (ignore 'MerkleHash')
        let mut arr = [0; 40];
        assert!(hex::decode_to_slice(res.rows[0].clone(), &mut arr).is_ok());
        let index = u64::from_le_bytes(arr[0..8].try_into().unwrap());

        Some(index)
    }
}

impl TokenContract
{
    pub fn new(endpoints: NonEmpty<String>) -> Self
    {
        TokenContract {
            endpoints,
            node_buffer: HashMap::new()
        }
    }

    pub async fn get_table_rows(
        &self,
        payload: &mut EOSGetTableRowsPayload
    ) -> EOSGetTableRowsResponse
    {
        let mut res = EOSGetTableRowsResponse{
            rows: Vec::new(),
            more: false,
            next_key: 0.to_string()
        };
        loop
        {
            // prepare POST request to fetch from EOSIO multiindex table
            let mut opts = RequestInit::new();
            opts.method("POST");
            opts.mode(RequestMode::Cors);
            opts.body(Some(&JsValue::from_str(&serde_json::to_string(payload).unwrap())));

            let url = format!("{}/v1/chain/get_table_rows", self.endpoints[0]);
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
            // {"rows":["", "", ...], "more": false, "next_key": ""}
            let tmp: EOSGetTableRowsResponse = serde_json::from_str(&str).unwrap();
            res.rows.extend(tmp.rows);

            // if there's more update payload struct and repeat
            if tmp.more
            {
                // if key type is not primary there can be more than one match per key. In order to
                // prevent an endless loop here (if next key equals lower bound of last payload) we
                // break the loop in that case. The number of desired results is defined by 'limit'.
                if payload.key_type != "primary" && payload.lower_bound != tmp.next_key
                {
                    payload.lower_bound = tmp.next_key;
                }
                else
                {
                    break;
                }
            }
            else
            {
                break;
            }
        }
        
        res
    }

    pub async fn get_merkle_hash(
        &self,
        index: u64
    ) -> Option<MerkleHashOrchard>
    {
        // prepare POST request to fetch from EOSIO multiindex table
        let payload = EOSGetTableRowsPayload{
            code: "thezeostoken".to_string(),
            table: "mteosram".to_string(),
            scope: "thezeostoken".to_string(),
            index_position: "primary".to_string(),
            key_type: "uint64_t".to_string(),
            encode_type: "hex".to_string(),
            lower_bound: index.to_string(),
            upper_bound: index.to_string(),
            limit: 1,
            reverse: false,
            show_payer: false
        };
        
        let res = self.get_table_rows(&mut payload.clone()).await;
        if res.rows.len() == 0
        {
            return None;
        }
        // extract serialized node data and parse 'MerkleHash' (ignore 'index')
        let mut arr = [0; 40];
        assert!(hex::decode_to_slice(res.rows[0].clone(), &mut arr).is_ok());
        let value = MerkleHashOrchard::from(Fp([
            u64::from_le_bytes(arr[ 8..16].try_into().unwrap()),
            u64::from_le_bytes(arr[16..24].try_into().unwrap()),
            u64::from_le_bytes(arr[24..32].try_into().unwrap()),
            u64::from_le_bytes(arr[32..40].try_into().unwrap())
        ]));
        
        Some(value)
    }

    pub async fn get_global_state(&self) -> Global
    {
        // prepare POST request to fetch from EOSIO singleton table
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);
        opts.body(Some(&JsValue::from_str("{\"code\":\"thezeostoken\",\"table\":\"global\",\"scope\":\"thezeostoken\"}")));

        let url = format!("{}/v1/chain/get_table_rows", self.endpoints[0]);
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

        let res: EOSGetTableRowsResponse = serde_json::from_str(&str).unwrap();
        if res.rows.is_empty()
        {
            return Global{
                note_count: 0,
                leaf_count: 0,
                tree_depth: 0,
            }
        }

        // parse serialized EOS data
        let mut arr = [0; 8+8+8];
        let str: String = res.rows[0].clone().chars().take((8+8+8)*2).collect();
        assert!(hex::decode_to_slice(str, &mut arr).is_ok());
        let note_count = u64::from_le_bytes(arr[0..8].try_into().unwrap());
        let leaf_count = u64::from_le_bytes(arr[8..16].try_into().unwrap());
        let tree_depth = u64::from_le_bytes(arr[16..24].try_into().unwrap());
        
        Global{
            note_count,
            leaf_count,
            tree_depth,
        }
    }

    pub async fn get_encrypted_notes(
        &self,
        from: u64,
        to: u64
    ) -> Vec<TransmittedNoteCiphertextEx>
    {
        // prepare POST request to fetch from EOSIO multiindex table
        let payload = EOSGetTableRowsPayload{
            code: "thezeostoken".to_string(),
            table: "noteseosram".to_string(),
            scope: "thezeostoken".to_string(),
            index_position: "primary".to_string(),
            key_type: "uint64_t".to_string(),
            encode_type: "dec".to_string(),
            lower_bound: from.to_string(),
            upper_bound: to.to_string(),
            limit: 10,
            reverse: false,
            show_payer: false
        };
        let res = self.get_table_rows(&mut payload.clone()).await;
        
        let mut v = Vec::new();
        for str in res.rows
        {
            // parse serialized EOS data
            let mut arr = [0; 8+8+1+32*2+2+ENC_CIPHERTEXT_SIZE*2+2+OUT_CIPHERTEXT_SIZE*2];
            assert!(hex::decode_to_slice(str, &mut arr).is_ok());
            let id = u64::from_le_bytes(arr[0..8].try_into().unwrap());
            let block_number = u64::from_le_bytes(arr[8..16].try_into().unwrap());
            // skip reading string sizes since we already know the exact length
            let epk_bytes: [u8; 32*2] = arr[24+1..24+1+32*2].try_into().unwrap();
            let enc_ciphertext: [u8; ENC_CIPHERTEXT_SIZE*2] = arr[24+1+32*2+2..24+1+32*2+2+ENC_CIPHERTEXT_SIZE*2].try_into().unwrap();
            let out_ciphertext: [u8; OUT_CIPHERTEXT_SIZE*2] = arr[24+1+32*2+2+ENC_CIPHERTEXT_SIZE*2+2..24+1+32*2+2+ENC_CIPHERTEXT_SIZE*2+2+OUT_CIPHERTEXT_SIZE*2].try_into().unwrap();
            let epk_bytes_str = String::from_utf8(epk_bytes.to_vec()).unwrap();
            let enc_ciphertext_str = String::from_utf8(enc_ciphertext.to_vec()).unwrap();
            let out_ciphertext_str = String::from_utf8(out_ciphertext.to_vec()).unwrap();
            let mut epk_bytes = [0; 32];
            let mut enc_ciphertext = [0; ENC_CIPHERTEXT_SIZE];
            let mut out_ciphertext = [0; OUT_CIPHERTEXT_SIZE];
            assert!(hex::decode_to_slice(epk_bytes_str, &mut epk_bytes).is_ok());
            assert!(hex::decode_to_slice(enc_ciphertext_str, &mut enc_ciphertext).is_ok());
            assert!(hex::decode_to_slice(out_ciphertext_str, &mut out_ciphertext).is_ok());
            v.push(TransmittedNoteCiphertextEx{
                id,
                block_number,
                encrypted_note: TransmittedNoteCiphertext{
                    epk_bytes,
                    enc_ciphertext,
                    out_ciphertext
                }
            });
        } 
        v
    }

    pub async fn get_currency_balance(
        &self,
        code: &String,
        account: &String,
        symbol: &String
    ) -> (u64, u8)
    {
        // prepare POST request for this API call
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);
        opts.body(Some(&JsValue::from_str(&format!("{{\"code\":\"{}\",\"account\":\"{}\",\"symbol\":\"{}\"}}", code, account, symbol))));

        let url = format!("{}/v1/chain/get_currency_balance", self.endpoints[0]);
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
        // ["257.2000 SYM"] or []
        if str == "[]" { return (0, 0); }
        let str = str.chars().skip(2).take(str.len()-4).collect::<String>();
        let dot = str.find('.');
        let ws = str.find(' ').unwrap(); // must contain whitespace
        let dec_len = if dot.is_some() { dot.unwrap() } else { ws };
        let frac_len = if dot.is_some() { ws-1 - dot.unwrap() } else { 0 };
        let dec = str.chars().take(dec_len).collect::<String>().parse::<u64>().unwrap() * 10_u64.pow(frac_len as u32);
        let frac = if dot.is_some() { str.chars().skip(dot.unwrap()+1).take(frac_len).collect::<String>().parse::<u64>().unwrap() } else { 0 };
        (dec + frac, frac_len as u8)
    }

    pub async fn get_nfts(
        &self,
        code: &String,
        account: &String
    ) -> Vec<(u64, String)>
    {
        // prepare POST request to fetch from EOSIO multiindex table
        let payload = EOSGetTableRowsPayload{
            code: code.clone(),
            table: "assets".to_string(),
            scope: account.clone(),
            index_position: "primary".to_string(),
            key_type: "uint64_t".to_string(),
            encode_type: "dec".to_string(),
            lower_bound: 0.to_string(),
            upper_bound: u64::MAX.to_string(),
            limit: 10,
            reverse: false,
            show_payer: false
        };
        let res = self.get_table_rows(&mut payload.clone()).await;
        
        let mut v = Vec::new();
        for str in res.rows
        {
            // parse serialized EOS data (only first two u64 values which is 'asset_id' and 'collection_name')
            let mut arr = [0; 8+8];
            let sub_str = str.chars().take(32).collect::<String>();
            assert!(hex::decode_to_slice(sub_str, &mut arr).is_ok());
            let id = u64::from_le_bytes(arr[0..8].try_into().unwrap());
            let collection = value_to_name(u64::from_le_bytes(arr[8..16].try_into().unwrap()));
            v.push((id, collection));
        }
        v
    }

    pub async fn upload_proof_to_liquidstorage(
        &self,
        proof: &String
    )
    {
        let fd = FormData::new().unwrap();
        assert!(fd.append_with_str("strupload", proof).is_ok());

        // prepare POST request to fetch from EOSIO multiindex table
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::NoCors); // TODO: should be CORS
        opts.body(Some(&fd));
        
        // TODO: Change to endpoints (need to have the 'web3uploader' service running)
        let url = "http://web3.zeos.one/uploadstr"; // TODO: should be DSP/ZEOS Validator
        let request = Request::new_with_str_and_init(&url, &opts).unwrap();
        
        // send http request using browser window's fetch
        let window = web_sys::window().unwrap();
        let _resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();

        // 'no-cors' mode doesn't allow the browser to read any response content.
        // see: https://stackoverflow.com/a/54906434/2340535
    }
}

#[cfg(test)]
mod tests
{
    use crate::tree::EMPTY_ROOTS;

    use super::MERKLE_DEPTH_ORCHARD;

    #[test]
    fn test_macros()
    {
        println!("{}", MT_ARR_FULL_TREE_OFFSET!(MERKLE_DEPTH_ORCHARD));
        println!("{}", MT_ARR_LEAF_ROW_OFFSET!(MERKLE_DEPTH_ORCHARD));
        println!("{}", MT_NUM_LEAVES!(MERKLE_DEPTH_ORCHARD));

        let array_index = 77;
        let tree_index = array_index / MT_ARR_FULL_TREE_OFFSET!(MERKLE_DEPTH_ORCHARD);
        let leaf_index = array_index - (tree_index + 1) * MT_ARR_FULL_TREE_OFFSET!(MERKLE_DEPTH_ORCHARD) + (tree_index + 1) * MT_NUM_LEAVES!(MERKLE_DEPTH_ORCHARD);
        println!("{}", leaf_index);

        println!("empty roots:");
        for r in EMPTY_ROOTS.iter()
        {
            println!("{}", hex::encode(r.inner().0[0].to_le_bytes()));
        }
    }
}