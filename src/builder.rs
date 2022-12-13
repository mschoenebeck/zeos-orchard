//! Logic for building entire EOSIO transactions depending on ZEOS privacy actions.

use crate::action::{RawZAction, ZA_MINTFT, ZA_MINTNFT, ZA_MINTAUTH, ZA_TRANSFERFT, ZA_TRANSFERNFT, ZA_BURNFT, ZA_BURNNFT, ZA_BURNAUTH};
use crate::address::Address;
use crate::tree::MerklePath;
use crate::note::{Note, Nullifier, NT_FT, NT_NFT, NT_AT};
use crate::keys::SpendingKey;
use crate::value::NoteValue;
use crate::note::ExtractedNoteCommitment;
use crate::keys::FullViewingKey;
use crate::bundle::Bundle;
use crate::contract::NoteEx;
use crate::eosio::name_to_value;

extern crate serde_json;

use rand::rngs::OsRng;
use rustzeos::halo2::{Proof, ProvingKey};
use sha256::digest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZActionDesc
{
    /// ...
    pub(crate) za_type: u64,
    pub(crate) to: String,  // EOS account for BURN actions or shielded address otherwise
    pub(crate) d1: String,  // u64
    pub(crate) d2: String,  // u64
    pub(crate) sc: String,
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

/// ...
pub trait HasMerkleTree
{
    /// fetches an entire merkle path asynchronously
    async fn get_merkle_path(
        &mut self,
        leaf_index: u64,
        leaf_count: u64,
    ) -> MerklePath;
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum TransactionBuilderError
{
    #[error("Error: {0}")]
    GeneralError(String)
}

impl From<halo2_proofs::plonk::Error> for TransactionBuilderError
{
    fn from(err: halo2_proofs::plonk::Error) -> Self
    {
        Self::GeneralError(err.to_string())
    }
}

/// ...
#[derive(Debug)]
pub struct TransactionBuilder
{
    leaf_count: u64
}

impl TransactionBuilder
{
    /// ...
    pub fn new(leaf_count: u64) -> Self
    {
        TransactionBuilder { leaf_count }
    }

    /// ...
    pub async fn build_transaction<D: HasMerkleTree>(
        &self,
        pk: &ProvingKey,
        sk: &SpendingKey,
        notes: &mut Vec<NoteEx>,
        action_descs: &Vec<EOSActionDesc>,
        contract: &mut D,
        eos_auth: &Vec<EOSAuthorization>
    ) -> Result<(Option<Proof>, Vec<EOSAction>), TransactionBuilderError>
    {
        let mut rng = OsRng.clone();

        // Walk through the whole list of action descriptors to detect the sequence of actions 
        // with privacy dependencies (aka zactions) within this transaction.
        let mut z_begin = -1;
        let mut z_end = -1;
        for i in 0..action_descs.len()
        {
            if action_descs[i].zaction_descs.len() > 0
            {
                z_end = i as i32;
                if z_begin == -1
                {
                    z_begin = i as i32;
                }
            }
        }

        // no zactions in this transaction => just return all EOSActions
        if z_begin == -1
        {
            let tx: Vec<EOSAction> = action_descs.iter().map(|ad| ad.action.clone()).collect();
            return Ok((None, tx));
        }

        // copy all EOS actions into the tx until the privacy sequence starts...
        let z_begin = z_begin as usize;
        let z_end = z_end as usize;
        let mut tx: Vec<EOSAction> = action_descs[0..z_begin].iter().map(|ad| ad.action.clone()).collect();
        
        // process 'step' actions of privacy sequence
        let mut list = Vec::new();
        let mut raw_zactions = Vec::new();
        for i in z_begin..=z_end
        {
            let mut rzactions_step = Vec::new();
            for zad in &action_descs[i].zaction_descs
            {
                // try to create vector of raw zactions from zaction descriptor
                let rzas = self.create_raw_zactions(sk, notes, zad, contract).await;
                if rzas.is_none()
                {
                    return Err(TransactionBuilderError::GeneralError(format!("ZActionDesc not executable: {}", serde_json::to_string(zad).unwrap())));
                }
                else
                {
                    rzactions_step.extend(rzas.unwrap());
                }
            }
            // if there are zactions for this step encode the zactions of all raw zactions of this step (including the dummy zaction!) into the EOS actions 'data'
            let mut a = action_descs[i].action.clone();
            if rzactions_step.len() > 0
            {
                let mut ser_zactions = format!("{:02X?}", rzactions_step.len() + 1);
                ser_zactions.push_str("efbeaddeefbeaddeefbeaddeefbeaddeefbeaddeefbeaddeefbeaddeefbeaddeefbeaddeefbeadde000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
                for rza in &rzactions_step
                {
                    ser_zactions.push_str(&rza.zaction().serialize_eos());
                }
                // append the already existing serialized data from before
                ser_zactions.push_str(&action_descs[i].action.data);
                a.data = ser_zactions;
            }
            list.push(a);
            // add the raw zactions of this step to the list of all raw zactions
            raw_zactions.extend(rzactions_step);
        }

        // process 'begin' action of privacy sequence
        let ((proof, _, _), _, encrypted_notes) = Bundle::from_parts(raw_zactions).prepare(pk, &mut rng)?;
        let data_str = format!("{{\"proof\":\"{}\",\"notes\":{},\"tx\":{}}}", 
            get_liquidstorage_uri(hex::encode(proof.as_ref()), true),
            serde_json::to_string(&encrypted_notes).unwrap(),
            serde_json::to_string(&list).unwrap()
        );

        // add 'begin' and 'step' actions to transaction
        tx.push(EOSAction{
            account: String::from("thezeostoken"),
            name: String::from("begin"),
            authorization: eos_auth.clone(),
            data: data_str
        });
        tx.extend(vec![EOSAction{
            account: String::from("thezeostoken"),
            name: String::from("step"),
            authorization: eos_auth.clone(),
            data: String::from("{}")
        }; list.len()]);

        // copy all EOS actions into the tx after the privacy sequence (if any)
        if action_descs.len() >= z_end
        {
            tx.extend(action_descs[z_end+1..].iter().map(|ad| ad.action.clone()).collect::<Vec<EOSAction>>());
        }

        Ok((Some(proof), tx))
    }

    /// Create as many raw ZActions as needed in order to execute the action described by 'desc' using the pool of 'notes'.
    /// Returns 'None' if the action described by 'desc' cannot be executed.
    pub async fn create_raw_zactions<D: HasMerkleTree>(
        &self,
        sk: &SpendingKey, 
        notes: &mut Vec<NoteEx>, 
        desc: &ZActionDesc, 
        contract: &mut D
    ) -> Option<Vec<RawZAction>>
    {
        let mut rng = OsRng.clone();
        let mut res = Vec::new();
        let fvk = FullViewingKey::from(sk);

        match desc.za_type
        {
            ZA_MINTFT | ZA_MINTNFT | ZA_MINTAUTH => {
                let recipient = Address::from_bech32m(&desc.to).unwrap();
                let mut memo_arr = [0; 512];
                assert!(desc.memo.len() < 512);
                memo_arr[0..desc.memo.len()].clone_from_slice(desc.memo.as_bytes());
                let nft = if desc.za_type == ZA_MINTFT { 0 } else { 1 };
                let note_b = Note::new(
                    match desc.za_type { ZA_MINTFT => NT_FT, ZA_MINTNFT => NT_NFT, ZA_MINTAUTH => NT_AT, _ => 0 },
                    recipient, 
                    NoteValue::from_raw(desc.d1.parse().unwrap()),
                    NoteValue::from_raw(desc.d2.parse().unwrap()), 
                    NoteValue::from_raw(name_to_value(&desc.sc)), 
                    NoteValue::from_raw(nft),
                    Nullifier::dummy(&mut rng), 
                    rng, 
                    memo_arr);
                let rza = RawZAction::from_parts(
                    desc.za_type,
                    &fvk,
                    None,
                    None,
                    Some(note_b),
                    None,
                    String::from(""),
                    rng
                );
                res.push(rza);
            }
            ZA_BURNAUTH => {
                // in this case the note commitment value of the auth note is stored in the 'to' field of 'desc'
                let mut to_arr = [0; 32];
                assert!(desc.to.len() == 2 * 32);
                hex::decode_to_slice(desc.to.clone(), &mut to_arr).unwrap();
                let nc = ExtractedNoteCommitment::from_bytes(&to_arr).unwrap();
                match select_auth_note(notes, name_to_value(&desc.sc), nc) {
                    Some(spent_note) => {
                        let rza = RawZAction::from_parts(
                            desc.za_type,
                            &fvk,
                            None,
                            None,
                            Some(spent_note.note),
                            None,
                            String::from(""),
                            rng
                        );
                        res.push(rza);
                    },
                    None => return None
                }
            }
            ZA_TRANSFERFT | ZA_BURNFT => {
                match select_fungible_notes(notes, desc.d1.parse().unwrap(), desc.d2.parse().unwrap(), name_to_value(&desc.sc)) {
                    Some((spent_notes, change)) => {
                        let mut memo_arr = [0; 512];
                        let mut recipient = Address::dummy(&mut rng); // dummy in case of burn
                        if desc.za_type == ZA_TRANSFERFT
                        {
                            recipient = Address::from_bech32m(&desc.to).unwrap();
                            assert!(desc.memo.len() < 512);
                            memo_arr[0..desc.memo.len()].clone_from_slice(desc.memo.as_bytes());
                        }
                        else
                        {
                            // in case of burn note_b's memo field contains the receiving EOS account name's value
                            assert!(desc.to.len() <= 12);
                            memo_arr[0..8].clone_from_slice(&name_to_value(&desc.to).to_be_bytes());
                        }
                        for i in 0..spent_notes.len()
                        {
                            let note_b = Note::new(
                                NT_FT,
                                recipient, 
                                if i == spent_notes.len()-1 { NoteValue::from_raw(spent_notes[i].note.d1().inner() - change) } else { spent_notes[i].note.d1() },
                                spent_notes[i].note.d2(),
                                spent_notes[i].note.sc(),
                                NoteValue::from_raw(0),
                                spent_notes[i].note.nullifier(&fvk),
                                rng, 
                                memo_arr);
                            let note_c = Note::new(
                                NT_FT,
                                spent_notes[i].note.recipient(), 
                                if i == spent_notes.len()-1 { NoteValue::from_raw(change) } else { NoteValue::from_raw(0) },
                                spent_notes[i].note.d2(),
                                spent_notes[i].note.sc(),
                                NoteValue::from_raw(0),
                                spent_notes[i].note.nullifier(&fvk),
                                rng,
                                [0; 512]);
                            let rza = RawZAction::from_parts(
                                desc.za_type,
                                &fvk,
                                //Some(merkle_path.2(spent_notes[i].leaf_index, merkle_path.0, merkle_path.1)),
                                Some(contract.get_merkle_path(spent_notes[i].leaf_index, self.leaf_count).await),
                                Some(spent_notes[i].note),
                                Some(note_b),
                                Some(note_c),
                                if desc.za_type == ZA_BURNFT { desc.memo.clone() } else { String::from("") },
                                rng
                            );
                            res.push(rza);
                        }
                    },
                    None => return None
                }
            }
            ZA_TRANSFERNFT | ZA_BURNNFT => {
                match select_nonfungible_note(notes, desc.d1.parse().unwrap(), desc.d2.parse().unwrap(), name_to_value(&desc.sc)) {
                    Some(spent_note) => {
                        let mut memo_arr = [0; 512];
                        let mut recipient = Address::dummy(&mut rng);
                        if desc.za_type == ZA_TRANSFERNFT
                        {
                            recipient = Address::from_bech32m(&desc.to).unwrap();
                            assert!(desc.memo.len() < 512);
                            memo_arr[0..desc.memo.len()].clone_from_slice(desc.memo.as_bytes());
                        }
                        else
                        {
                            // in case of burn note_b's memo field contains the receiving EOS account name's value
                            assert!(desc.to.len() <= 12);
                            memo_arr[0..8].clone_from_slice(&name_to_value(&desc.to).to_be_bytes());
                        }
                        let note_b = Note::new(
                            NT_NFT,
                            recipient, 
                            spent_note.note.d1(),
                            spent_note.note.d2(),
                            spent_note.note.sc(),
                            NoteValue::from_raw(1),
                            spent_note.note.nullifier(&fvk),
                            rng, 
                            memo_arr);
                        let rza = RawZAction::from_parts(
                            desc.za_type,
                            &fvk,
                            //Some(merkle_path.2(spent_note.leaf_index, merkle_path.0, merkle_path.1)),
                            Some(contract.get_merkle_path(spent_note.leaf_index, self.leaf_count).await),
                            Some(spent_note.note),
                            Some(note_b),
                            None,
                            if desc.za_type == ZA_BURNNFT { desc.memo.clone() } else { String::from("") },
                            rng
                        );
                        res.push(rza);
                    },
                    None => return None
                }
            }
            _ => return None,
        }

        Some(res)
    }
}

/// Very simple note selection algorithm: walk through all notes and pick notes of the demanded type until the sum
/// is equal or greater than the requested 'amount'. Returns tuple of vector of notes to be spent and the change that
/// is left over from the last note. Returns 'None' if there are not enough notes to reach 'amount'.
pub fn select_fungible_notes(notes: &mut Vec<NoteEx>, amount: u64, symbol: u64, sc: u64) -> Option<(Vec<NoteEx>, u64)>
{
    // sort 'notes' by note value (d1), ascending order and walk backwards through them in order to be able to safely remove elements
    notes.sort_by(|a, b| a.note.d1().inner().cmp(&b.note.d1().inner()));
    let mut res = Vec::new();
    let mut sum = 0;
    for i in (0..notes.len()).rev()
    {
        if sc == notes[i].note.sc().inner()         // same smart contract
            && symbol == notes[i].note.d2().inner() // same symbol
            && notes[i].note.nft().inner() == 0     // fungible (not NFT)
        {
            sum += notes[i].note.d1().inner();
            res.push(notes.remove(i));
            if sum >= amount
            {
                return Some((res, sum - amount));
            }
        }
    }
    // Not enough notes! Move picked notes in 'res' back to 'notes' and return 'None'.
    notes.append(&mut res);
    None
}

/// Walk through all notes and look for the NFT. Return 'None' if not found.
pub fn select_nonfungible_note(notes: &mut Vec<NoteEx>, d1: u64, d2: u64, sc: u64) -> Option<NoteEx>
{
    for i in 0..notes.len()
    {
        if sc == notes[i].note.sc().inner()     // same smart contract
            && d2 == notes[i].note.d2().inner() // same d2
            && d1 == notes[i].note.d1().inner() // same d1
            && notes[i].note.nft().inner() != 0 // non-fungible (NFT)
        {
            return Some(notes.remove(i));
        }
    }
    None
}

/// Walk through all notes and look for the Auth NFT with a certain commitment value. Return 'None' if not found.
pub fn select_auth_note(notes: &mut Vec<NoteEx>, sc: u64, nc: ExtractedNoteCommitment) -> Option<NoteEx>
{
    for i in 0..notes.len()
    {
        if sc == notes[i].note.sc().inner()             // same smart contract
            && nc == notes[i].note.commitment().into()  // same commitment value
            && notes[i].note.nft().inner() != 0         // non-fungible (NFT)
        {
            return Some(notes.remove(i));
        }
    }
    None
}

/// calculate LiquidStorage URI used for the IPFS addressing of data
pub fn get_liquidstorage_uri(input: String, short: bool) -> String
{
    let protocol_str = if short { "z" } else { "ipfs://z" };
    format!("{}{}", protocol_str, bs58::encode(hex::decode(format!("{}{}", "01551220", digest(input))).unwrap()).into_string())
}

#[cfg(test)]
mod tests
{
    use rand::{rngs::OsRng, seq::SliceRandom};
    use crate::{note::NT_FT, note::NT_AT, tree::MerklePath, action::{ZA_TRANSFERFT, ZA_BURNFT, ZA_MINTFT, ZA_MINTNFT, ZA_MINTAUTH, ZA_TRANSFERNFT, ZA_BURNNFT, ZA_BURNAUTH}, keys::FullViewingKey, keys::Scope, note::ExtractedNoteCommitment, builder::get_liquidstorage_uri};
    use super::{select_fungible_notes, select_auth_note, select_nonfungible_note, TransactionBuilder, Note, NoteValue, Address, Nullifier, NoteEx, SpendingKey, EOSAction, HasMerkleTree};
    use super::name_to_value;
    use super::{ZActionDesc, EOSActionDesc, EOSAuthorization};
    use crate::circuit::{Circuit, K};
    use rustzeos::halo2::ProvingKey;

    #[test]
    fn test_liquidstorage_uri()
    {
        let val = get_liquidstorage_uri("hello".to_string(), false);
        assert_eq!(val, "ipfs://zb2rhZfjRh2FHHB2RkHVEvL2vJnCTcu7kwRqgVsf9gpkLgteo");
    }

    #[test]
    fn serde_note()
    {
        let mut rng = OsRng.clone();
        let (_, _, n) = Note::dummy(&mut rng, None, Some(NoteValue::from_raw(1844674407370955161)));

        let sn = serde_json::to_string(&n).unwrap();
        //println!("{}", sn);
        let dsn: Note = serde_json::from_str(&sn).unwrap();

        assert_eq!(dsn, n);
    }

    #[test]
    fn note_selection()
    {
        let mut rng = OsRng.clone();
        let mut notes = Vec::new();
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, Address::dummy(&mut rng), NoteValue::from_raw(5), NoteValue::from_raw(1), NoteValue::from_raw(1), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, Address::dummy(&mut rng), NoteValue::from_raw(3), NoteValue::from_raw(1), NoteValue::from_raw(1), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, Address::dummy(&mut rng), NoteValue::from_raw(2), NoteValue::from_raw(1), NoteValue::from_raw(1), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_AT, Address::dummy(&mut rng), NoteValue::from_raw(1337), NoteValue::from_raw(0), NoteValue::from_raw(111), NoteValue::from_raw(1), Nullifier::dummy(&mut rng), rng, [0; 512])});
        let nc = notes[3].note.commitment().into();

        let (mut spent_notes, change) = select_fungible_notes(&mut notes, 6, 1, 1).unwrap();
        assert_eq!(spent_notes.len(), 2);
        assert_eq!(change, 2);

        notes.append(&mut spent_notes);

        let auth_note = select_auth_note(&mut notes, 111, nc).unwrap();
        assert_eq!(auth_note.note.d1().inner(), 1337);

        notes.push(auth_note);
        notes.shuffle(&mut rng);

        let nft = select_nonfungible_note(&mut notes, 1337, 0, 111).unwrap();
        assert_eq!(nft.note.d1().inner(), 1337);
    }

    pub struct DummyContract;
    impl HasMerkleTree for DummyContract
    {
        async fn get_merkle_path(&mut self, _leaf_index: u64, _leaf_count: u64) -> MerklePath
        {
            let mut rng = OsRng.clone();
            MerklePath::dummy(&mut rng)
        }
    }

    #[tokio::test]
    async fn zaction_creation()
    {
        let mut rng = OsRng.clone();
        let mut notes = Vec::new();
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, Address::dummy(&mut rng), NoteValue::from_raw(5), NoteValue::from_raw(1), NoteValue::from_raw(name_to_value(&"thezeostoken".to_string())), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, Address::dummy(&mut rng), NoteValue::from_raw(3), NoteValue::from_raw(1), NoteValue::from_raw(name_to_value(&"thezeostoken".to_string())), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, Address::dummy(&mut rng), NoteValue::from_raw(2), NoteValue::from_raw(1), NoteValue::from_raw(name_to_value(&"thezeostoken".to_string())), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_AT, Address::dummy(&mut rng), NoteValue::from_raw(1337), NoteValue::from_raw(0), NoteValue::from_raw(name_to_value(&"nftzeostoken".to_string())), NoteValue::from_raw(1), Nullifier::dummy(&mut rng), rng, [0; 512])});
        let nc: ExtractedNoteCommitment = notes[3].note.commitment().into();

        let sk = SpendingKey::from_zip32_seed(b"miau seed miau 123 Der seed muss lang genug sein...", 0, 0).unwrap();
        let fvk: FullViewingKey = (&sk).into();

        let mut desc = ZActionDesc {
            za_type: ZA_MINTFT,
            to: fvk.address_at(0u32, Scope::External).to_bech32m(),
            d1: "6".to_string(),
            d2: "1".to_string(),
            sc: "thezeostoken".to_string(),
            memo: String::from("")
        };

        let tb = TransactionBuilder::new(0); // leaf_count not required for DummyContract's get_merkle_path()
        let mut dc = DummyContract;
        println!("{:?}", tb.create_raw_zactions(&sk, &mut notes.clone(), &desc, &mut dc).await.unwrap());
        desc.za_type = ZA_MINTNFT;
        println!("{:?}", tb.create_raw_zactions(&sk, &mut notes.clone(), &desc, &mut dc).await.unwrap());
        desc.za_type = ZA_MINTAUTH;
        println!("{:?}", tb.create_raw_zactions(&sk, &mut notes.clone(), &desc, &mut dc).await.unwrap());
        desc.za_type = ZA_TRANSFERFT;
        println!("{:?}", tb.create_raw_zactions(&sk, &mut notes.clone(), &desc, &mut dc).await.unwrap());
        desc.za_type = ZA_BURNFT;
        desc.to = String::from("mschoenebeck");
        println!("{:?}", tb.create_raw_zactions(&sk, &mut notes.clone(), &desc, &mut dc).await.unwrap());

        let mut desc = ZActionDesc {
            za_type: ZA_TRANSFERNFT, 
            to: fvk.address_at(0u32, Scope::External).to_bech32m(),
            d1: "1337".to_string(), 
            d2: "0".to_string(), 
            sc: "nftzeostoken".to_string(), 
            memo: String::from("")
        };
        println!("{:?}", tb.create_raw_zactions(&sk, &mut notes.clone(), &desc, &mut dc).await.unwrap());
        desc.za_type = ZA_BURNNFT;
        desc.to = String::from("mschoenebeck");
        println!("{:?}", tb.create_raw_zactions(&sk, &mut notes.clone(), &desc, &mut dc).await.unwrap());
        desc.za_type = ZA_BURNAUTH;
        desc.to = hex::encode(nc.to_bytes());
        println!("{:?}", tb.create_raw_zactions(&sk, &mut notes.clone(), &desc, &mut dc).await.unwrap());

    }

    #[tokio::test]
    async fn transaction_building()
    {
        let mut rng = OsRng.clone();
        
        let sk = SpendingKey::from_zip32_seed(b"miau seed miau 123 Der seed muss lang genug sein...", 0, 0).unwrap();
        let fvk: FullViewingKey = (&sk).into();
        
        let mut notes = Vec::new();
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, fvk.address_at(0u32, Scope::External), NoteValue::from_raw(5), NoteValue::from_raw(1), NoteValue::from_raw(name_to_value(&"thezeostoken".to_string())), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, fvk.address_at(0u32, Scope::External), NoteValue::from_raw(3), NoteValue::from_raw(1), NoteValue::from_raw(name_to_value(&"thezeostoken".to_string())), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, fvk.address_at(0u32, Scope::External), NoteValue::from_raw(2), NoteValue::from_raw(1), NoteValue::from_raw(name_to_value(&"thezeostoken".to_string())), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_AT, fvk.address_at(0u32, Scope::External), NoteValue::from_raw(1337), NoteValue::from_raw(0), NoteValue::from_raw(111), NoteValue::from_raw(1), Nullifier::dummy(&mut rng), rng, [0; 512])});
        let _nc: ExtractedNoteCommitment = notes[3].note.commitment().into();
        notes.push(NoteEx{id: 0, block_number: 0, leaf_index:0, note: Note::new(NT_FT, fvk.address_at(0u32, Scope::External), NoteValue::from_raw(10000), NoteValue::from_raw(1397703940), NoteValue::from_raw(6138663591592764928), NoteValue::from_raw(0), Nullifier::dummy(&mut rng), rng, [0; 512])});

        let newstock1dex_auth = [EOSAuthorization{actor: "newstock1dex".to_string(), permission: "active".to_string()}; 1];
        let thezeostoken_auth = [EOSAuthorization{actor: "thezeostoken".to_string(), permission: "active".to_string()}; 1];
        
        assert_eq!(6138663577826885632, name_to_value(&"eosio".to_string()));
        assert_eq!(6138663587900751872, name_to_value(&"eosio.msig".to_string()));
        assert_eq!(6138663591592764928, name_to_value(&"eosio.token".to_string()));
        assert_eq!(10813382581022265600, name_to_value(&"mschoenebeck".to_string()));

        let ad0 = EOSActionDesc{
            action: EOSAction{
                account: "eosio.token".to_string(),
                name: "transfer".to_string(),
                authorization: newstock1dex_auth.to_vec(),
                data: "{\"from\":\"newstock1dex\", \"to\":\"thezeostoken\", \"quantity\":\"1.0000 EOS\", \"memo\":\"unit test only\"}".to_string()
            }, 
            zaction_descs: Vec::new()
        };
        let ad1 = EOSActionDesc{
            action: EOSAction{
                account: "eosio.token".to_string(),
                name: "transfer".to_string(),
                authorization: newstock1dex_auth.to_vec(),
                data: "{\"from\":\"newstock1dex\", \"to\":\"thezeostoken\", \"quantity\":\"1.0000 EOS\", \"memo\":\"kylin test\"}".to_string()
            }, 
            zaction_descs: Vec::new()
        };
        let ad2 = EOSActionDesc{
            action: EOSAction{
                account: "thezeostoken".to_string(),
                name: "exec".to_string(),
                authorization: thezeostoken_auth.to_vec(),
                data: "".to_string()
            }, 
            zaction_descs: [
                ZActionDesc{
                    za_type: ZA_MINTFT,
                    to: fvk.address_at(0u32, Scope::External).to_bech32m(),
                    d1: "10000".to_string(),
                    d2: "1397703940".to_string(),
                    sc: "thezeostoken".to_string(),
                    memo: "This is a test!".to_string()
                }
            ].to_vec()
        };
        let ad3 = EOSActionDesc{
            action: EOSAction{
                account: "eosio.token".to_string(),
                name: "transfer".to_string(),
                authorization: newstock1dex_auth.to_vec(),
                data: "{\"from\":\"newstock1dex\", \"to\":\"thezeostoken\", \"quantity\":\"1.0000 EOS\", \"memo\":\"unit test only\"}".to_string()
            }, 
            zaction_descs: Vec::new()
        };
        let ad4 = EOSActionDesc{
            action: EOSAction{
                account: "thezeostoken".to_string(),
                name: "exec".to_string(),
                authorization: thezeostoken_auth.to_vec(),
                data: "".to_string()
            }, 
            zaction_descs: [
                ZActionDesc{
                    za_type: ZA_BURNFT,
                    to: "mschoenebeck".to_string(),
                    d1: "9".to_string(),
                    d2: "1".to_string(),
                    sc: "thezeostoken".to_string(),
                    memo: "transfer test".to_string()
                }
            ].to_vec()
        };
        let ad5 = EOSActionDesc{
            action: EOSAction{
                account: "eosio.token".to_string(),
                name: "transfer".to_string(),
                authorization: newstock1dex_auth.to_vec(),
                data: "{\"from\":\"newstock1dex\", \"to\":\"thezeostoken\", \"quantity\":\"1.0000 EOS\", \"memo\":\"unit test only\"}".to_string()
            }, 
            zaction_descs: Vec::new()
        };
        let ad6 = EOSActionDesc{
            action: EOSAction{
                account: "eosio.token".to_string(),
                name: "transfer".to_string(),
                authorization: newstock1dex_auth.to_vec(),
                data: "{\"from\":\"newstock1dex\", \"to\":\"thezeostoken\", \"quantity\":\"1.0000 EOS\", \"memo\":\"unit test only\"}".to_string()
            }, 
            zaction_descs: Vec::new()
        };
        
        let mut action_descs = Vec::new();
        action_descs.push(ad0);
        action_descs.push(ad1);
        action_descs.push(ad2);
        action_descs.push(ad3);
        action_descs.push(ad4);
        action_descs.push(ad5);
        action_descs.push(ad6);
        
        //use rustzeos::halo2::VerifyingKey;
        //use crate::circuit::{Circuit, K};
        //use std::fs::File;
        //use std::io::prelude::*;
        //let vk = VerifyingKey::build(Circuit::default(), K);
        //let mut arr = Vec::new();
        //vk.serialize(&mut arr);
        //println!("{}", hex::encode(arr));
        //let mut file = File::create("vk.txt").unwrap();
        //write!(file, "{}", hex::encode(arr).to_uppercase());
        
        let tb = TransactionBuilder::new(0); // leaf_count not required for DummyContract's get_merkle_path()
        let mut dc = DummyContract;
        let pk = ProvingKey::build(Circuit::default(), K);
        let (proof, actions) = tb.build_transaction(
            &pk,
            &sk,
            &mut notes,
            &action_descs,
            &mut dc,
            &newstock1dex_auth.to_vec()
        ).await.unwrap();

        // print transaction data for manual execution of transactions
        println!("{}", serde_json::to_string(&actions).unwrap());
        println!("{}", hex::encode(proof.clone().unwrap()));

        //let mut inputs = Vec::new();
        //hex::decode_to_slice(actions[1].data, &mut inputs);
        //assert!(zeos_verifier::verify_zeos_proof(proof.unwrap().as_ref(), &inputs, &arr));
    }

}
