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
#![deny(unsafe_code)]

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
use action::{ZA_MINTFT, ZA_MINTNFT, ZA_MINTAUTH, ZA_TRANSFERFT, ZA_TRANSFERNFT, ZA_BURNFT, ZA_BURNNFT, ZA_BURNAUTH};
pub use address::Address;
pub use bundle::Bundle;
pub use note::Note;
use rustzeos::halo2::Proof;
pub use tree::Anchor;

use note::TransmittedNoteCiphertext;
use tree::MerklePath;
use crate::note::{Nullifier, RandomSeed};
use crate::note_encryption::ENC_CIPHERTEXT_SIZE;
use crate::keys::SpendingKey;
use crate::value::NoteValue;
use crate::note::ExtractedNoteCommitment;
use crate::keys::FullViewingKey;
use crate::note::NoteCommitment;

extern crate serde_json;

use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use std::fmt;

use rand::rngs::OsRng;

#[macro_use]
extern crate serde_derive;


impl Serialize for TransmittedNoteCiphertext
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 8 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("TransmittedNoteCiphertext", 3)?;
        state.serialize_field("epk_bytes", &&hex::encode(self.epk_bytes))?;
        state.serialize_field("enc_ciphertext", &hex::encode(self.enc_ciphertext))?;
        state.serialize_field("out_ciphertext", &hex::encode(self.out_ciphertext))?;
        state.end()
    }
}

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
        // 8 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Note", 8)?;
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
        enum Field { Recipient, D1, D2, Sc, Nft, Rho, Rseed, Memo }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`recipient` or `d1` or `d2` or `sc` or `nft` or `rho` or `rseed` or `memo`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
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
                let recipient_str: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let mut recipient = [0; 43];
                assert_eq!(hex::decode_to_slice(recipient_str, &mut recipient), Ok(()));
                let recipient = Address::from_raw_address_bytes(&recipient).unwrap();
                let d1: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let d1 = d1.parse::<u64>().unwrap();
                let d1 = NoteValue::from_raw(d1);
                let d2: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let d2 = d2.parse::<u64>().unwrap();
                let d2 = NoteValue::from_raw(d2);
                let sc: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let sc = sc.parse::<u64>().unwrap();
                let sc = NoteValue::from_raw(sc);
                let nft = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let nft = NoteValue::from_raw(nft);
                let rho = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;
                let rho = Nullifier::from_bytes(&rho).unwrap();
                let rseed = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?;
                let rseed = RandomSeed::from_bytes(rseed, &rho).unwrap();
                let memo_str: String = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?;
                let mut memo = [0; 512];
                assert_eq!(hex::decode_to_slice(memo_str, &mut memo), Ok(()));
                Ok(Note::from_parts(recipient, d1, d2, sc, nft, rho, rseed, memo))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Note, V::Error>
            where
                V: MapAccess<'de>,
            {
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

                Ok(Note::from_parts(recipient, d1, d2, sc, nft, rho, rseed, memo))
            }
        }

        const FIELDS: &'static [&'static str] = &["recipient", "d1", "d2", "sc", "nft", "rho", "rseed", "memo"];
        deserializer.deserialize_struct("Note", FIELDS, NoteVisitor)
    }
}

impl EOSNote
{
    pub fn from_parts(bn: u64, li: u64, note: Note) -> Self
    {
        EOSNote { block_number: bn.to_string(), leaf_index: li.to_string(), note }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EOSAuthorization
{
    pub(crate) actor: String,
    pub(crate) permission: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EOSAction
{
    pub(crate) account: String,
    pub(crate) name: String,
    pub(crate) authorization: Vec<EOSAuthorization>,
    /// JSON string (unpacked EOSAction) or HEX string (packed EOSAction)
    pub(crate) data: String,
}

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

