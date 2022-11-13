use crate::{
    tree::MerklePath,
    note::{Note, ExtractedNoteCommitment, Nullifier, TransmittedNoteCiphertext},
    primitives::redpallas::VerificationKey,
    value::NoteValue, keys::{FullViewingKey, SpendValidatingKey}, circuit::Instance, Anchor, note_encryption::NoteEncryption,
    keys::Scope::External,
    note_encryption::OrchardDomain
};
use pasta_curves::pallas;
use rand::RngCore;
use ff::Field;
use group::GroupEncoding;
use group::Curve;
use pasta_curves::arithmetic::CurveAffine;

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

    /// Returns the type of this zaction
    pub fn za_type(&self) -> u64
    {
        self.za_type
    }

    /// serialize EOS
    pub fn serialize_eos(&self) -> String
    {
        let mut res = String::from(hex::encode(self.za_type.to_le_bytes()));
        res.push_str(&hex::encode(self.ins.anchor.inner().0[0].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.anchor.inner().0[1].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.anchor.inner().0[2].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.anchor.inner().0[3].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.nf.inner().0[0].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.nf.inner().0[1].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.nf.inner().0[2].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.nf.inner().0[3].to_le_bytes()));
        let rk = pallas::Point::from_bytes(&self.ins.rk.clone().into())
            .unwrap()
            .to_affine()
            .coordinates()
            .unwrap();
        let x = rk.x();
        let y = rk.y();
        res.push_str(&hex::encode(x.0[0].to_le_bytes()));
        res.push_str(&hex::encode(x.0[1].to_le_bytes()));
        res.push_str(&hex::encode(x.0[2].to_le_bytes()));
        res.push_str(&hex::encode(x.0[3].to_le_bytes()));
        res.push_str(&hex::encode(y.0[0].to_le_bytes()));
        res.push_str(&hex::encode(y.0[1].to_le_bytes()));
        res.push_str(&hex::encode(y.0[2].to_le_bytes()));
        res.push_str(&hex::encode(y.0[3].to_le_bytes()));
        res.push_str(if self.ins.nft {"01"} else {"00"});
        res.push_str(&hex::encode(self.ins.b_d1.inner().to_le_bytes()));
        res.push_str(&hex::encode(self.ins.b_d2.inner().to_le_bytes()));
        res.push_str(&hex::encode(self.ins.b_sc.inner().to_le_bytes()));
        res.push_str(&hex::encode(self.ins.c_d1.inner().to_le_bytes()));
        res.push_str(&hex::encode(self.ins.cmb.inner().0[0].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.cmb.inner().0[1].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.cmb.inner().0[2].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.cmb.inner().0[3].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.cmc.inner().0[0].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.cmc.inner().0[1].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.cmc.inner().0[2].to_le_bytes()));
        res.push_str(&hex::encode(self.ins.cmc.inner().0[3].to_le_bytes()));
        let memo = if self.memo.len() >= 255 {&self.memo.as_bytes()[0..255]} else {self.memo.as_bytes()};
        let len = format!("{:02X?}", memo.len());
        res.push_str(&len);
        res.push_str(&hex::encode(memo));

        res
    }
}

/// An action applied to the global ledger.
///
/// ...
#[derive(Debug, Clone)]
pub struct RawZAction
{
    za_type: u64,
    fvk: FullViewingKey,
    auth_path_a: Option<MerklePath>,
    alpha_a: pallas::Scalar,
    note_a: Option<Note>,
    note_b: Option<Note>,
    note_c: Option<Note>,
    memo: String
}

impl RawZAction
{
    /// Constructs a `RawZAction` from its constituent parts.
    pub fn from_parts<R: RngCore>(
        za_type: u64,
        fvk: &FullViewingKey,
        auth_path_a: Option<MerklePath>,
        note_a: Option<Note>,
        note_b: Option<Note>,
        note_c: Option<Note>,
        memo: String,
        mut rng: R
    ) -> Self {

        // TODO checks

        RawZAction {
            za_type,
            fvk: fvk.clone(),
            auth_path_a,
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

    /// Returns full viewing key
    pub fn fvk(&self) -> FullViewingKey
    {
        self.fvk.clone()
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
        let nft = self.za_type == ZA_MINTNFT || self.za_type == ZA_TRANSFERNFT || self.za_type == ZA_BURNNFT || self.za_type == ZA_MINTAUTH || self.za_type == ZA_BURNAUTH;
        let mut b_d1 = NoteValue::from_raw(0);
        let mut b_d2 = NoteValue::from_raw(0);
        let mut b_sc = NoteValue::from_raw(0);
        let mut c_d1 = NoteValue::from_raw(0);
        let mut cmb = ExtractedNoteCommitment::from(pallas::Base::zero());
        let mut cmc = ExtractedNoteCommitment::from(pallas::Base::zero());

        if self.note_a.is_some()
        {
            anchor = self.auth_path_a.as_ref().unwrap().root(self.note_a.unwrap().commitment().into());
            nf = self.note_a.unwrap().nullifier(&self.fvk);
            let ak: SpendValidatingKey = self.fvk.clone().into();
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
            if self.za_type == ZA_BURNFT || self.za_type == ZA_BURNNFT || self.za_type == ZA_BURNFT2
            {
                // TODO: set receiving EOS account name from notes memo field
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
                // TODO: set receiving EOS account from notes memo field
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
        if self.za_type != ZA_BURNFT && self.za_type != ZA_BURNFT2 && self.za_type != ZA_BURNNFT && self.za_type != ZA_BURNAUTH
        {
            // encrypt note_b
            let ne = NoteEncryption::new(Some(self.fvk.to_ovk(External)), self.note_b.unwrap());
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
            // encrypt note_c
            // TODO: if note_c.d1 == 0 add dummy note
            let ne = NoteEncryption::new(Some(self.fvk.to_ovk(External)), self.note_c.unwrap());
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

#[cfg(test)]
mod tests
{
    use super::{RawZAction, Note};
    use crate::OsRng;

    #[test]
    fn eos_serialization()
    {
        let mut rng = OsRng.clone();
        let (sk, fvk, note) = Note::dummy(&mut rng, None, None);
        let rza = RawZAction::from_parts(0xDEADBEEFDEADBEEF, &fvk, None, None, Some(note), None, String::from("mschoenebeck"), rng);
        println!("{}", rza.zaction().serialize_eos());
    }
}