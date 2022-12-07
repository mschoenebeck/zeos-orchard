//! Logic for everything wallet related.

use crate::builder::TransactionBuilder;
use crate::constants::MERKLE_DEPTH_ORCHARD;
use crate::keys::{PreparedIncomingViewingKey, SpendingKey, FullViewingKey, Scope::External};
use crate::contract::{Global, NoteEx, TokenContract};
use crate::{ENDPOINTS};
use crate::circuit::{Circuit, K};

use rustzeos::halo2::ProvingKey;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
extern crate serde_json;
extern crate bitcoin_bech32;
use bitcoin_bech32::u5;
use std::collections::HashMap;

/// Wallet settings
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings
{
    /// list of API endpoints
    eos_endpoints: Vec<String>,
    dsp_endpoints: Vec<String>,
    zeos_endpoints: Vec<String>,
    /// maps token symbol to contract name and decimals of known fungible tokens
    ft_contracts: HashMap<String, (String, u8)>,
    /// list of known NFT contracts
    nft_contracts: Vec<String>
}

impl Default for Settings
{
    fn default() -> Self
    {
        Settings {
            eos_endpoints: vec![
                "https://kylin.eosn.io".to_string()
            ],
            dsp_endpoints: vec![
                "https://kylin-dsp-1.liquidapps.io".to_string()
            ],
            zeos_endpoints: vec![
            ],
            ft_contracts: HashMap::from([
                ("EOS".to_string(), ("eosio.token".to_string(), 4)),
                ("DAPP".to_string(), ("dappservices".to_string(), 4)),
                ("ZEOS".to_string(), ("thezeostoken".to_string(), 4)),
            ]),
            nft_contracts: vec![
                "atomicassets".to_string()
            ],
        }
    }
}

/// A ZEOS wallet.
#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet
{
    /// The seed phrase
    pub(crate) seed: String,
    /// The state of this wallet
    pub(crate) state: Global,
    /// The settings of this wallet
    pub(crate) settings: Settings,
    /// The zk-SNARK proving key
    #[serde(skip)]
    #[serde(default = "default_proving_key")]
    pk: ProvingKey,
    /// The received/spendable notes of this wallet
    pub(crate) spendable_notes: Vec<NoteEx>,
    /// The notes that have been sent from this wallet
    pub(crate) sent_notes: Vec<NoteEx>,
}

fn default_proving_key() -> ProvingKey
{
    ProvingKey::build(Circuit::default(), K)
}

#[wasm_bindgen]
impl Wallet
{
    /// Creates a new wallet from seed phrase
    /// TODO: add 'wallet birthday' (i.e. allow for initialization of 'state' as well)
    pub fn new(seed: String) -> Self
    {
        assert!(SpendingKey::from_zip32_seed(seed.as_bytes(), 0, 0).is_ok());
        Wallet {
            seed: seed.clone(),
            state: Global{ note_count: 0, leaf_count: 0, tree_depth: MERKLE_DEPTH_ORCHARD as u64 },
            settings: Settings::default(),
            pk: default_proving_key(),
            spendable_notes: Vec::new(),
            sent_notes: Vec::new(),
        }
    }

    /// Restores a wallet from JSON string
    pub fn restore(json: String) -> Self
    {
        let res: Self = serde_json::from_str(&json).unwrap();
        res
    }

    /// Converts a wallet to JSON formatted string to be restored later using
    /// the 'restore' function above.
    pub fn to_json_string(&self) -> String
    {
        serde_json::to_string(self).unwrap()
    }

    /// Returns the address of a certain diversifier as hex string
    pub fn address(
        &self,
        diversifier_index: u32
    ) -> String
    {
        let sk = SpendingKey::from_zip32_seed(self.seed.as_bytes(), 0, 0).unwrap();
        let fvk = FullViewingKey::from(&sk);
        let addr = fvk.address_at(diversifier_index, External);
        hex::encode(addr.to_raw_address_bytes())
    }

    /// Synchronize wallet state with contract state
    pub async fn sync(&mut self)
    {
        let mut contract = TokenContract::new(ENDPOINTS.map(String::from));
        let global = contract.get_global_state().await;
        if global.note_count == self.state.note_count
        {
            return;
        }

        // derive keys required to decrypt notes
        let fvk = FullViewingKey::from(&SpendingKey::from_zip32_seed(self.seed.as_bytes(), 0, 0).unwrap());

        let encrypted_notes = contract.get_encrypted_notes(self.state.note_count, global.note_count).await;
        let mut new_notes = Vec::new();
        for en in encrypted_notes
        {
            let o = en.try_decrypt_as_receiver(&PreparedIncomingViewingKey::new(&fvk.to_ivk(External)));
            if o.is_some()
            {
                new_notes.push(o.unwrap());
            }
            let o = en.try_decrypt_as_sender(&fvk.to_ovk(External));
            if o.is_some()
            {
                let sn = o.unwrap();
                self.spendable_notes.retain(|n| n.note.nullifier(&fvk) != sn.note.rho());
                self.sent_notes.push(sn);
            }
        }

        // determine leaf_indices of 'new_notes'
        for i in 0..new_notes.len()
        {
            contract.determine_leaf_index(&mut new_notes[i], global.leaf_count).await;
        }

        // move new notes into 'notes' and update wallet state
        self.spendable_notes.append(&mut new_notes);
        self.state = global;
    }

    pub async fn create_transaction(
        &self,
        js_action_descs: String,   // Vec<EOSActionDesc>
        js_eos_auth: String        // Vec<EOSAuth>
    ) -> String
    {
        let action_descs = serde_json::from_str(&js_action_descs).unwrap();
        let eos_auth = serde_json::from_str(&js_eos_auth).unwrap();
        let mut contract = TokenContract::new(ENDPOINTS.map(String::from));
        let builder = TransactionBuilder::new(self.state.leaf_count);
        let sk = SpendingKey::from_zip32_seed(self.seed.as_bytes(), 0, 0).unwrap();
        //log(&format!("{:?}", action_descs));

        let (proof, actions) = builder.build_transaction(
            &self.pk,
            &sk,
            &mut self.spendable_notes.clone(),
            &action_descs,
            &mut contract,
            &eos_auth
        ).await;

        assert!(proof.is_some());
        let proof_str = hex::encode(proof.unwrap().as_ref());
        //contract.upload_proof_to_liquidstorage(&proof_str).await;

        // return JSON string of EOS actions ready to execute
        // all non-serialized 'data' strings should be valid JSON
        // => remove quotation marks and backslashes
        serde_json::to_string(&actions).unwrap()
            .replace(r#""data":"{"#, r#""data":{"#)
            .replace(r#"}"}"#, r#"}}"#)
            .replace("\\", "")
    }
}

#[cfg(test)]
mod tests
{


    #[test]
    fn test_regex()
    {
        
    }
}
