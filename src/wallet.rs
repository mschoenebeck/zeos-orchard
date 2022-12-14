//! Logic for everything wallet related.

use crate::builder::TransactionBuilder;
use crate::constants::MERKLE_DEPTH_ORCHARD;
use crate::keys::{PreparedIncomingViewingKey, SpendingKey, FullViewingKey, Scope::External};
use crate::contract::{Global, NoteEx, TokenContract};
use crate::{ENDPOINTS, log};
use crate::circuit::{Circuit, K};
use crate::eosio::{symbol_to_string_precision, string_to_symbol, value_to_name};

use rustzeos::halo2::ProvingKey;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
extern crate serde_json;
use std::collections::HashMap;
use std::fmt::Debug;

/// Wallet settings
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings
{
    /// list of API endpoints
    eos_endpoints: Vec<String>,
    dsp_endpoints: Vec<String>,
    zeos_endpoints: Vec<String>,
    /// maps token symbol to contract name and decimals of known fungible tokens
    ft_contracts: HashMap<String, (String, u64)>,
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
                ("EOS".to_string(), ("eosio.token".to_string(), string_to_symbol(&"EOS".to_string(), 4))),
                ("DAPP".to_string(), ("dappservices".to_string(), string_to_symbol(&"DAPP".to_string(), 4))),
                ("ZEOS".to_string(), ("thezeostoken".to_string(), string_to_symbol(&"ZEOS".to_string(), 4))),
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
    /// The internal diversifier index indicates how many addresses have been derived for this wallet
    pub(crate) diversifier_index: u32,
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

impl From<crate::zip32::Error> for JsError
{
    fn from(err: crate::zip32::Error) -> Self
    {
        JsError::new(&err.to_string())
    }
}

#[wasm_bindgen]
impl Wallet
{
    /// Creates a new wallet from seed phrase
    /// TODO: add 'wallet birthday' (i.e. allow for initialization of 'state' as well)
    pub fn new(seed: String) -> Result<Wallet, JsError>
    {
        SpendingKey::from_zip32_seed(seed.as_bytes(), 0, 0)?;
        Ok(Wallet {
            seed: seed.clone(),
            state: Global{ note_count: 0, leaf_count: 0, tree_depth: MERKLE_DEPTH_ORCHARD as u64 },
            settings: Settings::default(),
            diversifier_index: 0,
            pk: default_proving_key(),
            spendable_notes: Vec::new(),
            sent_notes: Vec::new(),
        })
    }

    /// Restores a wallet from JSON string
    pub fn from_string(json: String) -> Result<Wallet, JsError>
    {
        let res: Self = serde_json::from_str(&json)?;
        Ok(res)
    }

    /// Converts a wallet to JSON formatted string to be restored later using
    /// the 'from_string' function above.
    pub fn to_string(&self) -> Result<String, JsError>
    {
        let res = serde_json::to_string(self)?;
        Ok(res)
    }

    /// Synchronize wallet state with contract state
    pub async fn sync(&mut self) -> Result<(), JsError>
    {
        let mut contract = TokenContract::new(ENDPOINTS.map(String::from));
        let global = contract.get_global_state().await;
        if global.note_count == self.state.note_count
        {
            return Ok(());
        }

        // derive keys required to decrypt notes
        let fvk = FullViewingKey::from(&SpendingKey::from_zip32_seed(self.seed.as_bytes(), 0, 0)?);

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

        Ok(())
    }

    pub async fn create_transaction(
        &self,
        js_action_descs: JsValue,   // Vec<EOSActionDesc>
        js_eos_auth: JsValue        // Vec<EOSAuth>
    ) -> Result<String, JsError>
    {
        let action_descs = serde_wasm_bindgen::from_value(js_action_descs)?;
        let eos_auth = serde_wasm_bindgen::from_value(js_eos_auth)?;
        let mut contract = TokenContract::new(ENDPOINTS.map(String::from));
        let builder = TransactionBuilder::new(self.state.leaf_count);
        let sk = SpendingKey::from_zip32_seed(self.seed.as_bytes(), 0, 0)?;
        //log(&format!("{:?}", action_descs));

        let (proof, actions) = builder.build_transaction(
            &self.pk,
            &sk,
            &mut self.spendable_notes.clone(),
            &action_descs,
            &mut contract,
            &eos_auth
        ).await?;

        assert!(proof.is_some());
        let proof_str = hex::encode(proof.unwrap().as_ref());
        contract.upload_proof_to_liquidstorage(&proof_str).await;

        // Returns JSON string of EOS actions ready to execute.
        // All non-serialized 'data' strings should be valid JSON
        // => remove quotation marks and backslashes
        Ok(serde_json::to_string(&actions)?
            .replace(r#""data":"{"#, r#""data":{"#)
            .replace(r#"}"}"#, r#"}}"#)
            .replace("\\", "")
        )
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
        addr.to_bech32m()
    }

    /// Increments the internal diversifier index by one and returns a newly derived wallet address from that index
    pub fn derive_new_wallet_address(&mut self) -> String
    {
        self.diversifier_index += 1;
        let sk = SpendingKey::from_zip32_seed(self.seed.as_bytes(), 0, 0).unwrap();
        let fvk = FullViewingKey::from(&sk);
        let addr = fvk.address_at(self.diversifier_index, External);
        addr.to_bech32m()
    }

    /// Returns a key/value map of all ever generated wallet addresses (diversifier_index => address)
    pub fn get_addresses(&self) -> JsValue
    {
        let mut map = HashMap::new();
        let sk = SpendingKey::from_zip32_seed(self.seed.as_bytes(), 0, 0).unwrap();
        let fvk = FullViewingKey::from(&sk);
        for i in 0..self.diversifier_index
        {
            map.insert(i, fvk.address_at(i, External).to_bech32m());
        }
        serde_wasm_bindgen::to_value(&map).unwrap()
    }

    /// Returns a key/value map of all fungible token balances of this wallet (symbol => balance)
    pub fn get_balances(&self) -> JsValue
    {
        let mut map = HashMap::new();
        for n in self.spendable_notes.iter()
        {
            if n.note.nft().inner() == 0
            {
                let symbol = symbol_to_string_precision(n.note.d2().inner()).0;
                let value = n.note.d1().inner();
                // check if token contract is correct
                if n.note.sc().inner() == self.settings.ft_contracts[&symbol].1
                {
                    map.entry(symbol).and_modify(|v| *v += value).or_insert(value);
                }
            }
        }
        serde_wasm_bindgen::to_value(&map).unwrap()
    }

    /// Returns a key/value map of all non-fungible token ids of this wallet (contract => array of id)
    pub fn get_nfts(&self) -> JsValue
    {
        let mut map = HashMap::new();
        for n in self.spendable_notes.iter()
        {
            if n.note.nft().inner() != 0
            {
                let contract = value_to_name(n.note.sc().inner());
                let mut id = vec![n.note.d1().inner()];
                map.entry(contract).and_modify(|v: &mut Vec<u64>| (*v).append(&mut id)).or_insert(id);
            }
        }
        serde_wasm_bindgen::to_value(&map).unwrap()
    }

    /// ...
    pub async fn get_account_balances(&self, account: String) -> JsValue
    {
        let contract = TokenContract::new(ENDPOINTS.map(String::from));
        let mut map = HashMap::new();
        for (sym, (code, _)) in self.settings.ft_contracts.iter()
        {
            let (amount, _) = contract.get_currency_balance(code, &account, sym).await;
            map.insert(sym.clone(), amount.to_string());
        }
        for (k,v) in map.iter()
        {
            log(&format!("{k}: {v}"));
        }
        serde_wasm_bindgen::to_value(&map).unwrap()
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
