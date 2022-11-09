use crate::{
    tree::MerklePath,
    note::{Note, ExtractedNoteCommitment, Nullifier, TransmittedNoteCiphertext, NoteCommitment},
    primitives::redpallas::{self, SpendAuth, VerificationKey},
    value::NoteValue, keys::{FullViewingKey, SpendValidatingKey}, circuit::Instance, Anchor, note_encryption::NoteEncryption,
    keys::Scope::External,
    note_encryption::OrchardDomain
};
use pasta_curves::pallas;
use rand::{rngs::OsRng, RngCore};
use ff::Field;

// ZEOS action types (must equal enum values in zeosio.hpp)
pub const ZA_DUMMY: u64         = 0xDEADBEEFDEADBEEF;   // dummy action that indicates zactions to be validated/executed
pub const ZA_NULL: u64          = 0x0;                  // NULL OP - do nothing (verify proof only)
pub const ZA_MINTFT: u64        = 0x1;
pub const ZA_MINTNFT: u64       = 0x2;
pub const ZA_MINTAUTH: u64      = 0x3;
pub const ZA_TRANSFERFT: u64    = 0x4;
pub const ZA_TRANSFERNFT: u64   = 0x5;
pub const ZA_BURNFT: u64        = 0x6;
pub const ZA_BURNFT2: u64       = 0x7;
pub const ZA_BURNNFT: u64       = 0x8;
pub const ZA_BURNAUTH: u64      = 0x9;

// ZEOS ZAction (See equivalent struct 'zaction' in zeosio.hpp)
#[derive(Debug)]
pub struct ZAction
{
    za_type: u64,
    ins: Instance,
    memo: String
}

impl ZAction
{
    /// Constructs an `Action` from its constituent parts.
    pub fn from_parts(za_type: u64, ins: Instance, memo: String) -> Self
    {
        ZAction{
            za_type,
            ins,
            memo
        }
    }

    /// Returns the instance (aka public inputs) of this zaction
    pub fn instance(&self) -> Instance
    {
        self.ins.clone()
    }
}

/// An action applied to the global ledger.
///
/// ...
#[derive(Debug, Clone)]
pub struct Action
{
    za_type: u64,
    auth_path_a: Option<MerklePath>,
    fvk_a: Option<FullViewingKey>,
    alpha_a: pallas::Scalar,
    note_a: Option<Note>,
    note_b: Option<Note>,
    note_c: Option<Note>,
    memo: String
}

impl Action
{
    /// Constructs an `Action` from its constituent parts.
    pub fn from_parts<R: RngCore>(
        za_type: u64,
        auth_path_a: Option<MerklePath>,
        fvk_a: Option<FullViewingKey>,
        note_a: Option<Note>,
        note_b: Option<Note>,
        note_c: Option<Note>,
        memo: String,
        mut rng: R
    ) -> Self {

        // TODO checks

        Action {
            za_type,
            auth_path_a,
            fvk_a,
            alpha_a: pallas::Scalar::random(&mut rng),
            note_a,
            note_b,
            note_c,
            memo
        }
    }

    /// Returns the zaction type of this action
    pub fn za_type(&self) -> u64
    {
        self.za_type
    }

    /// Returns auth path of note a
    pub fn auth_path_a(&self) -> Option<&MerklePath>
    {
        self.auth_path_a.as_ref()
    }

    /// Returns full viewing key of note a
    pub fn fvk_a(&self) -> Option<&FullViewingKey>
    {
        self.fvk_a.as_ref()
    }

    /// Returns alpha of note a
    pub fn alpha_a(&self) -> pallas::Scalar
    {
        self.alpha_a
    }

    /// Returns note a
    pub fn note_a(&self) -> Option<Note>
    {
        self.note_a
    }

    /// Returns note b
    pub fn note_b(&self) -> Option<Note>
    {
        self.note_b
    }

    /// Returns note c
    pub fn note_c(&self) -> Option<Note>
    {
        self.note_c
    }

    /// returns the corresponding ZAction
    pub fn zaction(&self) -> ZAction
    {
        let mut anchor = Anchor::from(pallas::Base::zero());
        let mut nf = Nullifier::from(pallas::Base::zero());
        let mut rk = VerificationKey::dummy();
        let nft = self.za_type == ZA_MINTNFT || self.za_type == ZA_TRANSFERNFT || self.za_type == ZA_BURNNFT;
        let mut b_d1 = NoteValue::from_raw(0);
        let mut b_d2 = NoteValue::from_raw(0);
        let mut b_sc = NoteValue::from_raw(0);
        let mut c_d1 = NoteValue::from_raw(0);
        let mut cmb = ExtractedNoteCommitment::from(pallas::Base::zero());
        let mut cmc = ExtractedNoteCommitment::from(pallas::Base::zero());

        if self.note_a.is_some()
        {
            anchor = self.auth_path_a.as_ref().unwrap().root(self.note_a.unwrap().commitment().into());
            nf = self.note_a.unwrap().nullifier(&self.fvk_a.as_ref().unwrap());
            let ak: SpendValidatingKey = self.fvk_a.clone().unwrap().into();
            rk = ak.randomize(&self.alpha_a);
        }
        if self.note_b.is_some()
        {
            if self.za_type != ZA_TRANSFERFT && self.za_type != ZA_TRANSFERNFT
            {
                b_d1 = self.note_b.unwrap().d1();
                b_d2 = self.note_b.unwrap().d2();
                b_sc = self.note_b.unwrap().sc();
            }
            if self.za_type != ZA_BURNFT && self.za_type != ZA_BURNFT2 && self.za_type != ZA_BURNNFT
            {
                cmb = self.note_b.unwrap().commitment().into();
            }
        }
        if self.note_c.is_some()
        {
            if self.za_type == ZA_BURNFT2
            {
                c_d1 = self.note_c.unwrap().d1();
            }
            if self.za_type == ZA_TRANSFERFT || self.za_type == ZA_BURNFT
            {
                cmc = self.note_c.unwrap().commitment().into();
            }
        }

        let ins = Instance::from_parts(anchor, nf, rk, nft, b_d1, b_d2, b_sc, c_d1, cmb, cmc);
        ZAction::from_parts(self.za_type, ins, self.memo.clone())
    }

    /// returns the encrypted notes 
    pub fn encrypted_notes<R: RngCore>(&self, mut rng: R) -> Vec<TransmittedNoteCiphertext>
    {
        let mut encrypted_notes = Vec::new();
        if self.za_type != ZA_BURNFT && self.za_type != ZA_BURNFT2 && self.za_type != ZA_BURNNFT
        {
            // encrypt note_b if > 0
            // if == 0 add dummy note
            // TODO
            let ne = NoteEncryption::new(Some(self.fvk_a.as_ref().unwrap().to_ovk(External)), self.note_b.unwrap());
            let esk = OrchardDomain::derive_esk(&self.note_b.unwrap()).unwrap();
            let epk = OrchardDomain::ka_derive_public(&self.note_b.unwrap(), &esk);
            let encrypted_note = TransmittedNoteCiphertext {
                epk_bytes: epk.to_bytes().0,
                enc_ciphertext: ne.encrypt_note_plaintext(),
                out_ciphertext: ne.encrypt_outgoing_plaintext(&mut rng),
            };
            encrypted_notes.push(encrypted_note);
        }
        if self.za_type == ZA_TRANSFERFT || self.za_type == ZA_BURNFT
        {
            // encrypt note_c if > 0
            // if == 0 add dummy note
            // TODO
            let ne = NoteEncryption::new(Some(self.fvk_a.as_ref().unwrap().to_ovk(External)), self.note_c.unwrap());
            let esk = OrchardDomain::derive_esk(&self.note_c.unwrap()).unwrap();
            let epk = OrchardDomain::ka_derive_public(&self.note_c.unwrap(), &esk);
            let encrypted_note = TransmittedNoteCiphertext {
                epk_bytes: epk.to_bytes().0,
                enc_ciphertext: ne.encrypt_note_plaintext(),
                out_ciphertext: ne.encrypt_outgoing_plaintext(&mut rng),
            };
            encrypted_notes.push(encrypted_note);
        }
        encrypted_notes
    }
}
