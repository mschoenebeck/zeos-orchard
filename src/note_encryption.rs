//! Note encryption for Zcash transactions.
//!
//! This crate implements the [in-band secret distribution scheme] for the Sapling and
//! Orchard protocols. It provides reusable methods that implement common note encryption
//! and trial decryption logic, and enforce protocol-agnostic verification requirements.
//!
//! Protocol-specific logic is handled via the [`Domain`] trait. Implementations of this
//! trait are provided in the [`zcash_primitives`] (for Sapling) and [`orchard`] crates;
//! users with their own existing types can similarly implement the trait themselves.
//!
//! [in-band secret distribution scheme]: https://zips.z.cash/protocol/protocol.pdf#saplingandorchardinband
//! [`zcash_primitives`]: https://crates.io/crates/zcash_primitives
//! [`orchard`]: https://crates.io/crates/orchard

//#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Catch documentation errors caused by code changes.
#![deny(broken_intra_doc_links)]
#![deny(unsafe_code)]
// TODO: #![deny(missing_docs)]

//#[cfg(feature = "alloc")]
//extern crate alloc;
//#[cfg(feature = "alloc")]
//use alloc::vec::Vec;

use core::convert::TryInto;

//use chacha20::{
//    cipher::{NewCipher, StreamCipher, StreamCipherSeek},
//    ChaCha20,
//};
use chacha20poly1305::{
    aead::{AeadInPlace, NewAead},
    ChaCha20Poly1305,
};

use crate::note::TransmittedNoteCiphertext;

use rand_core::RngCore;
use subtle::{Choice, ConstantTimeEq};

/// The size of [`NotePlaintextBytes`].
pub const NOTE_PLAINTEXT_SIZE: usize = 1 + // version
    8  + // header
    11 + // diversifier
    8  + // d1
    8  + // d2
    8  + // sc
    8  + // nft
    32 + // rho
    32 + // rseed (or rcm prior to ZIP 212)
    512; // memo
/// The size of [`OutPlaintextBytes`].
pub const OUT_PLAINTEXT_SIZE: usize = 32 + // pk_d
    32; // esk
const AEAD_TAG_SIZE: usize = 16;
/// The size of an encrypted note plaintext.
pub const ENC_CIPHERTEXT_SIZE: usize = NOTE_PLAINTEXT_SIZE + AEAD_TAG_SIZE;
/// The size of an encrypted outgoing plaintext.
pub const OUT_CIPHERTEXT_SIZE: usize = OUT_PLAINTEXT_SIZE + AEAD_TAG_SIZE;

/// A symmetric key that can be used to recover a single Sapling or Orchard output.
#[derive(Debug)]
pub struct OutgoingCipherKey(pub [u8; 32]);

impl From<[u8; 32]> for OutgoingCipherKey {
    fn from(ock: [u8; 32]) -> Self {
        OutgoingCipherKey(ock)
    }
}

impl AsRef<[u8]> for OutgoingCipherKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Newtype representing the byte encoding of an [`EphemeralPublicKey`].
///
/// [`EphemeralPublicKey`]: Domain::EphemeralPublicKey
#[derive(Clone, Debug)]
pub struct EphemeralKeyBytes(pub [u8; 32]);

impl AsRef<[u8]> for EphemeralKeyBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for EphemeralKeyBytes {
    fn from(value: [u8; 32]) -> EphemeralKeyBytes {
        EphemeralKeyBytes(value)
    }
}

impl ConstantTimeEq for EphemeralKeyBytes {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

/// Newtype representing the byte encoding of a note plaintext.
#[derive(Debug)]
pub struct NotePlaintextBytes(pub [u8; NOTE_PLAINTEXT_SIZE]);
/// Newtype representing the byte encoding of a outgoing plaintext.
#[derive(Debug)]
pub struct OutPlaintextBytes(pub [u8; OUT_PLAINTEXT_SIZE]);

#[derive(Copy, Clone, PartialEq, Eq)]
enum NoteValidity {
    Valid,
    Invalid,
}

/// Implementation of in-band secret distribution for Orchard bundles.
/// A struct containing context required for encrypting Orchard notes.
///
/// This struct provides a safe API for encrypting Orchard notes. In particular, it
/// enforces that fresh ephemeral keys are used for every note, and that the ciphertexts are
/// consistent with each other.
///
/// Implements section 4.19 of the
/// [Zcash Protocol Specification](https://zips.z.cash/protocol/nu5.pdf#saplingandorchardinband)
#[derive(Debug)]
pub struct NoteEncryption {
    epk: EphemeralPublicKey,
    esk: EphemeralSecretKey,
    note: Note,
    /// `None` represents the `ovk = ⊥` case.
    ovk: Option<OutgoingViewingKey>,
}

impl NoteEncryption {
    /// Construct a new note encryption context for the specified note,
    /// recipient, and memo.
    pub fn new(
        ovk: Option<OutgoingViewingKey>,
        note: Note,
    ) -> Self {
        let esk = OrchardDomain::derive_esk(&note).expect("ZIP 212 is active.");
        NoteEncryption {
            epk: OrchardDomain::ka_derive_public(&note, &esk),
            esk,
            note,
            ovk,
        }
    }

    /// Exposes the ephemeral secret key being used to encrypt this note.
    pub fn esk(&self) -> &EphemeralSecretKey {
        &self.esk
    }

    /// Exposes the encoding of the ephemeral public key being used to encrypt this note.
    pub fn epk(&self) -> &EphemeralPublicKey {
        &self.epk
    }

    /// Generates `encCiphertext` for this note.
    pub fn encrypt_note_plaintext(&self) -> [u8; ENC_CIPHERTEXT_SIZE] {
        let pk_d = OrchardDomain::get_pk_d(&self.note);
        let shared_secret = OrchardDomain::ka_agree_enc(&self.esk, &pk_d);
        let key = OrchardDomain::kdf(shared_secret, &OrchardDomain::epk_bytes(&self.epk));
        let input = OrchardDomain::note_plaintext_bytes(&self.note);

        let mut output = [0u8; ENC_CIPHERTEXT_SIZE];
        output[..NOTE_PLAINTEXT_SIZE].copy_from_slice(&input.0);
        let tag = ChaCha20Poly1305::new(key.as_ref().into())
            .encrypt_in_place_detached(
                [0u8; 12][..].into(),
                &[],
                &mut output[..NOTE_PLAINTEXT_SIZE],
            )
            .unwrap();
        output[NOTE_PLAINTEXT_SIZE..].copy_from_slice(&tag);

        output
    }

    /// Generates `outCiphertext` for this note.
    pub fn encrypt_outgoing_plaintext<R: RngCore>(
        &self,
        rng: &mut R,
    ) -> [u8; OUT_CIPHERTEXT_SIZE] {
        let (ock, input) = if let Some(ovk) = &self.ovk {
            let ock = OrchardDomain::derive_ock(ovk, &OrchardDomain::epk_bytes(&self.epk));
            let input = OrchardDomain::outgoing_plaintext_bytes(&self.note, &self.esk);

            (ock, input)
        } else {
            // ovk = ⊥
            let mut ock = OutgoingCipherKey([0; 32]);
            let mut input = [0u8; OUT_PLAINTEXT_SIZE];

            rng.fill_bytes(&mut ock.0);
            rng.fill_bytes(&mut input);

            (ock, OutPlaintextBytes(input))
        };

        let mut output = [0u8; OUT_CIPHERTEXT_SIZE];
        output[..OUT_PLAINTEXT_SIZE].copy_from_slice(&input.0);
        let tag = ChaCha20Poly1305::new(ock.as_ref().into())
            .encrypt_in_place_detached([0u8; 12][..].into(), &[], &mut output[..OUT_PLAINTEXT_SIZE])
            .unwrap();
        output[OUT_PLAINTEXT_SIZE..].copy_from_slice(&tag);

        output
    }
}

/// Trial decryption of the full note plaintext by the recipient.
///
/// Attempts to decrypt and validate the given shielded output using the given `ivk`.
/// If successful, the corresponding note and memo are returned, along with the address to
/// which the note was sent.
///
/// Implements section 4.19.2 of the
/// [Zcash Protocol Specification](https://zips.z.cash/protocol/nu5.pdf#decryptivk).
pub fn try_note_decryption(
    ivk: &PreparedIncomingViewingKey,
    encrypted_note: &TransmittedNoteCiphertext,
) -> Option<Note> {
    let ephemeral_key = EphemeralKeyBytes(encrypted_note.epk_bytes);

    let epk = OrchardDomain::prepare_epk(OrchardDomain::epk(&ephemeral_key)?);
    let shared_secret = OrchardDomain::ka_agree_dec(ivk, &epk);
    let key = OrchardDomain::kdf(shared_secret, &ephemeral_key);

    try_note_decryption_inner(ivk, &ephemeral_key, encrypted_note, key)
}

fn try_note_decryption_inner(
    ivk: &PreparedIncomingViewingKey,
    ephemeral_key: &EphemeralKeyBytes,
    encrypted_note: &TransmittedNoteCiphertext,
    key: Hash,
) -> Option<Note> {
    let enc_ciphertext = encrypted_note.enc_ciphertext;

    let mut plaintext =
        NotePlaintextBytes(enc_ciphertext[..NOTE_PLAINTEXT_SIZE].try_into().unwrap());

    ChaCha20Poly1305::new(key.as_ref().into())
        .decrypt_in_place_detached(
            [0u8; 12][..].into(),
            &[],
            &mut plaintext.0,
            enc_ciphertext[NOTE_PLAINTEXT_SIZE..].into(),
        )
        .ok()?;

    let note = parse_note_plaintext_ivk(
        ivk,
        ephemeral_key,
        &plaintext.0,
    )?;

    Some(note)
}

fn parse_note_plaintext_ivk(
    ivk: &PreparedIncomingViewingKey,
    ephemeral_key: &EphemeralKeyBytes,
    plaintext: &[u8],
) -> Option<Note> {
    let note = OrchardDomain::parse_note_plaintext_ivk(ivk, &plaintext)?;

    if let NoteValidity::Valid = check_note_validity(&note, ephemeral_key) {
        Some(note)
    } else {
        None
    }
}

fn check_note_validity(
    note: &Note,
    ephemeral_key: &EphemeralKeyBytes,
) -> NoteValidity {
    if let Some(derived_esk) = OrchardDomain::derive_esk(note) {
        if OrchardDomain::epk_bytes(&OrchardDomain::ka_derive_public(&note, &derived_esk))
            .ct_eq(&ephemeral_key)
            .into()
        {
            NoteValidity::Valid
        } else {
            NoteValidity::Invalid
        }
    } else {
        // Before ZIP 212
        NoteValidity::Valid
    }
}

/// Recovery of the full note plaintext by the sender.
///
/// Attempts to decrypt and validate the given shielded output using the given `ovk`.
/// If successful, the corresponding note and memo are returned, along with the address to
/// which the note was sent.
///
/// Implements [Zcash Protocol Specification section 4.19.3][decryptovk].
///
/// [decryptovk]: https://zips.z.cash/protocol/nu5.pdf#decryptovk
pub fn try_output_recovery_with_ovk(
    ovk: &OutgoingViewingKey,
    encrypted_note: &TransmittedNoteCiphertext,
) -> Option<Note> {
    let ock = OrchardDomain::derive_ock(ovk, &EphemeralKeyBytes(encrypted_note.epk_bytes));
    try_output_recovery_with_ock(&ock, encrypted_note, &encrypted_note.out_ciphertext)
}

/// Recovery of the full note plaintext by the sender.
///
/// Attempts to decrypt and validate the given shielded output using the given `ock`.
/// If successful, the corresponding note and memo are returned, along with the address to
/// which the note was sent.
///
/// Implements part of section 4.19.3 of the
/// [Zcash Protocol Specification](https://zips.z.cash/protocol/nu5.pdf#decryptovk).
/// For decryption using a Full Viewing Key see [`try_output_recovery_with_ovk`].
pub fn try_output_recovery_with_ock(
    ock: &OutgoingCipherKey,
    encrypted_note: &TransmittedNoteCiphertext,
    out_ciphertext: &[u8; OUT_CIPHERTEXT_SIZE],
) -> Option<Note> {
    let enc_ciphertext = encrypted_note.enc_ciphertext;

    let mut op = OutPlaintextBytes([0; OUT_PLAINTEXT_SIZE]);
    op.0.copy_from_slice(&out_ciphertext[..OUT_PLAINTEXT_SIZE]);

    ChaCha20Poly1305::new(ock.as_ref().into())
        .decrypt_in_place_detached(
            [0u8; 12][..].into(),
            &[],
            &mut op.0,
            out_ciphertext[OUT_PLAINTEXT_SIZE..].into(),
        )
        .ok()?;

    let pk_d = OrchardDomain::extract_pk_d(&op)?;
    let esk = OrchardDomain::extract_esk(&op)?;

    let ephemeral_key = EphemeralKeyBytes(encrypted_note.epk_bytes);
    let shared_secret = OrchardDomain::ka_agree_enc(&esk, &pk_d);
    // The small-order point check at the point of output parsing rejects
    // non-canonical encodings, so reencoding here for the KDF should
    // be okay.
    let key = OrchardDomain::kdf(shared_secret, &ephemeral_key);

    let mut plaintext = NotePlaintextBytes([0; NOTE_PLAINTEXT_SIZE]);
    plaintext
        .0
        .copy_from_slice(&enc_ciphertext[..NOTE_PLAINTEXT_SIZE]);

    ChaCha20Poly1305::new(key.as_ref().into())
        .decrypt_in_place_detached(
            [0u8; 12][..].into(),
            &[],
            &mut plaintext.0,
            enc_ciphertext[NOTE_PLAINTEXT_SIZE..].into(),
        )
        .ok()?;

    let note = OrchardDomain::parse_note_plaintext_ovk(&pk_d, &esk, &ephemeral_key, &plaintext)?;

    // ZIP 212: Check that the esk provided to this function is consistent with the esk we
    // can derive from the note.
    if let Some(derived_esk) = OrchardDomain::derive_esk(&note) {
        if (!derived_esk.ct_eq(&esk)).into() {
            return None;
        }
    }

    if let NoteValidity::Valid =
        check_note_validity(&note, &ephemeral_key)
    {
        Some(note)
    } else {
        None
    }
}




// ! In-band secret distribution for Orchard bundles.
use blake2b_simd::{Hash, Params};
use group::ff::PrimeField;

use crate::{
    keys::{
        DiversifiedTransmissionKey, Diversifier, EphemeralPublicKey, EphemeralSecretKey,
        IncomingViewingKey, OutgoingViewingKey, PreparedEphemeralPublicKey, PreparedIncomingViewingKey, SharedSecret,
    },
    note::{ExtractedNoteCommitment, Nullifier, RandomSeed},
    spec::diversify_hash,
    value::{NoteValue},
    Address, Note,
};

const PRF_OCK_ORCHARD_PERSONALIZATION: &[u8; 16] = b"Zcash_Orchardock";

/// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][concreteprfs].
///
/// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
pub(crate) fn prf_ock_orchard(
    ovk: &OutgoingViewingKey,
    ephemeral_key: &EphemeralKeyBytes,
) -> OutgoingCipherKey {
    OutgoingCipherKey(
        Params::new()
            .hash_length(32)
            .personal(PRF_OCK_ORCHARD_PERSONALIZATION)
            .to_state()
            .update(ovk.as_ref())
            .update(ephemeral_key.as_ref())
            .finalize()
            .as_bytes()
            .try_into()
            .unwrap(),
    )
}

fn orchard_parse_note_plaintext<F>(
    plaintext: &[u8],
    get_validated_pk_d: F,
) -> Option<Note>
where
    F: FnOnce(&Diversifier) -> Option<DiversifiedTransmissionKey>,
{
    assert!(plaintext.len() == NOTE_PLAINTEXT_SIZE);

    // Check note plaintext version
    if plaintext[0] != 0x02 {
        return None;
    }

    // The unwraps below are guaranteed to succeed by the assertion above
    let header = u64::from_le_bytes(plaintext[1..9].try_into().unwrap());
    let diversifier = Diversifier::from_bytes(plaintext[9..20].try_into().unwrap());
    let d1 = NoteValue::from_bytes(plaintext[20..28].try_into().unwrap());
    let d2 = NoteValue::from_bytes(plaintext[28..36].try_into().unwrap());
    let sc = NoteValue::from_bytes(plaintext[36..44].try_into().unwrap());
    let nft = NoteValue::from_bytes(plaintext[44..52].try_into().unwrap());
    let rho = Nullifier::from_bytes(plaintext[52..84].try_into().unwrap()).unwrap();
    let rseed = Option::from(RandomSeed::from_bytes(
        plaintext[84..116].try_into().unwrap(),
        &rho,
    ))?;
    let memo = plaintext[116..NOTE_PLAINTEXT_SIZE].try_into().unwrap();

    let pk_d = get_validated_pk_d(&diversifier)?;

    let recipient = Address::from_parts(diversifier, pk_d);
//<<<<<<< HEAD
//    let note = Note::from_parts(header, recipient, d1, d2, sc, nft, rho, rseed, memo);
//    Some(note)
//=======
//    let note = Option::from(Note::from_parts(recipient, value, domain.rho, rseed))?;
//    Some((note, recipient))
//>>>>>>> d05b6cee9df7c4019509e2f54899b5979fb641b5
    Option::from(Note::from_parts(header, recipient, d1, d2, sc, nft, rho, rseed, memo).unwrap())
}

/// Orchard-specific note encryption logic.
#[derive(Debug)]
pub struct OrchardDomain {
}
/*
impl memuse::DynamicUsage for OrchardDomain {
    fn dynamic_usage(&self) -> usize {
        self.rho.dynamic_usage()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        self.rho.dynamic_usage_bounds()
    }
}
*/
impl OrchardDomain {
//<<<<<<< HEAD
    /// Derives the `EphemeralSecretKey` corresponding to this note.
    ///
    /// Returns `None` if the note was created prior to [ZIP 212], and doesn't have a
    /// deterministic `EphemeralSecretKey`.
    ///
    /// [ZIP 212]: https://zips.z.cash/zip-0212
    pub fn derive_esk(note: &Note) -> Option<EphemeralSecretKey> {
/*=======
    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_action<T>(act: &Action<T>) -> Self {
        OrchardDomain {
            rho: *act.nullifier(),
        }
    }

    /// Constructs a domain from a nullifier.
    pub fn for_nullifier(nullifier: Nullifier) -> Self {
        OrchardDomain { rho: nullifier }
    }
}

impl Domain for OrchardDomain {
    type EphemeralSecretKey = EphemeralSecretKey;
    type EphemeralPublicKey = EphemeralPublicKey;
    type PreparedEphemeralPublicKey = PreparedEphemeralPublicKey;
    type SharedSecret = SharedSecret;
    type SymmetricKey = Hash;
    type Note = Note;
    type Recipient = Address;
    type DiversifiedTransmissionKey = DiversifiedTransmissionKey;
    type IncomingViewingKey = PreparedIncomingViewingKey;
    type OutgoingViewingKey = OutgoingViewingKey;
    type ValueCommitment = ValueCommitment;
    type ExtractedCommitment = ExtractedNoteCommitment;
    type ExtractedCommitmentBytes = [u8; 32];
    type Memo = [u8; 512]; // TODO use a more interesting type

    fn derive_esk(note: &Self::Note) -> Option<Self::EphemeralSecretKey> {
>>>>>>> d05b6cee9df7c4019509e2f54899b5979fb641b5*/
        Some(note.esk())
    }

    /// Extracts the `DiversifiedTransmissionKey` from the note.
    fn get_pk_d(note: &Note) -> DiversifiedTransmissionKey {
        *note.recipient().pk_d()
    }
    
    fn prepare_epk(epk: EphemeralPublicKey) -> PreparedEphemeralPublicKey {
        PreparedEphemeralPublicKey::new(epk)
    }

//<<<<<<< HEAD
    /// Derives `EphemeralPublicKey` from `esk` and the note's diversifier.
    pub fn ka_derive_public(
        note: &Note,
        esk: &EphemeralSecretKey,
    ) -> EphemeralPublicKey {
/*=======
    fn prepare_epk(epk: Self::EphemeralPublicKey) -> Self::PreparedEphemeralPublicKey {
        PreparedEphemeralPublicKey::new(epk)
    }

    fn ka_derive_public(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> Self::EphemeralPublicKey {
>>>>>>> d05b6cee9df7c4019509e2f54899b5979fb641b5*/
        esk.derive_public(note.recipient().g_d())
    }

    /// Derives the `SharedSecret` from the sender's information during note encryption.
    fn ka_agree_enc(
        esk: &EphemeralSecretKey,
        pk_d: &DiversifiedTransmissionKey,
    ) -> SharedSecret {
        esk.agree(pk_d)
    }

    /// Derives the `SharedSecret` from the recipient's information during note trial
    /// decryption.
    fn ka_agree_dec(
//<<<<<<< HEAD
        ivk: &PreparedIncomingViewingKey,
        epk: &PreparedEphemeralPublicKey
    ) -> SharedSecret {
//=======
//        ivk: &Self::IncomingViewingKey,
//        epk: &Self::PreparedEphemeralPublicKey,
//    ) -> Self::SharedSecret {
//>>>>>>> d05b6cee9df7c4019509e2f54899b5979fb641b5
        epk.agree(ivk)
    }

    /// Derives the `SymmetricKey` used to encrypt the note plaintext.
    ///
    /// `secret` is the `SharedSecret` obtained from [`Self::ka_agree_enc`] or
    /// [`Self::ka_agree_dec`].
    ///
    /// `ephemeral_key` is the byte encoding of the [`EphemeralPublicKey`] used to derive
    /// `secret`. During encryption it is derived via [`Self::epk_bytes`]; during trial
    /// decryption it is obtained from [`ShieldedOutput::ephemeral_key`].
    ///
    /// [`EphemeralPublicKey`]: Self::EphemeralPublicKey
    /// [`EphemeralSecretKey`]: Self::EphemeralSecretKey
    fn kdf(secret: SharedSecret, ephemeral_key: &EphemeralKeyBytes) -> Hash {
        secret.kdf_orchard(ephemeral_key)
    }

    /// Encodes the given `Note` and `Memo` as a note plaintext.
    ///
    /// [`zcash_primitives` has been refactored]: https://github.com/zcash/librustzcash/issues/454
    fn note_plaintext_bytes(
        note: &Note,
    ) -> NotePlaintextBytes {
        let mut np = [0; NOTE_PLAINTEXT_SIZE];
        np[0] = 0x02;
        np[1..9].copy_from_slice(&note.header().to_le_bytes());
        np[9..20].copy_from_slice(note.recipient().diversifier().as_array());
        np[20..28].copy_from_slice(&note.d1().to_bytes());
        np[28..36].copy_from_slice(&note.d2().to_bytes());
        np[36..44].copy_from_slice(&note.sc().to_bytes());
        np[44..52].copy_from_slice(&note.nft().to_bytes());
        np[52..84].copy_from_slice(&note.rho().to_bytes());
        np[84..116].copy_from_slice(note.rseed().as_bytes());
        np[116..].copy_from_slice(&note.memo());
        NotePlaintextBytes(np)
    }

    /// Derives the [`OutgoingCipherKey`] for an encrypted note, given the note-specific
    /// public data and an `OutgoingViewingKey`.
    fn derive_ock(
        ovk: &OutgoingViewingKey,
        ephemeral_key: &EphemeralKeyBytes,
    ) -> OutgoingCipherKey {
        prf_ock_orchard(ovk, ephemeral_key)
    }

    /// Encodes the outgoing plaintext for the given note.
    fn outgoing_plaintext_bytes(
        note: &Note,
        esk: &EphemeralSecretKey,
    ) -> OutPlaintextBytes {
        let mut op = [0; OUT_PLAINTEXT_SIZE];
        op[..32].copy_from_slice(&note.recipient().pk_d().to_bytes());
        op[32..].copy_from_slice(&esk.0.to_repr());
        OutPlaintextBytes(op)
    }

    /// Returns the byte encoding of the given `EphemeralPublicKey`.
    fn epk_bytes(epk: &EphemeralPublicKey) -> EphemeralKeyBytes {
        epk.to_bytes()
    }

    /// Attempts to parse `ephemeral_key` as an `EphemeralPublicKey`.
    ///
    /// Returns `None` if `ephemeral_key` is not a valid byte encoding of an
    /// `EphemeralPublicKey`.
    fn epk(ephemeral_key: &EphemeralKeyBytes) -> Option<EphemeralPublicKey> {
        EphemeralPublicKey::from_bytes(&ephemeral_key.0).into()
    }

    /// Derives the `ExtractedCommitment` for this note.
    fn cmstar(note: &Note) -> ExtractedNoteCommitment {
        note.commitment().into()
    }

    /// Parses the given note plaintext from the recipient's perspective.
    ///
    /// The implementation of this method must check that:
    /// - The note plaintext version is valid (for the given decryption domain's context,
    ///   which may be passed via `self`).
    /// - The note plaintext contains valid encodings of its various fields.
    /// - Any domain-specific requirements are satisfied.
    ///
    /// `&self` is passed here to enable the implementation to enforce contextual checks,
    /// such as rules like [ZIP 212] that become active at a specific block height.
    ///
    /// [ZIP 212]: https://zips.z.cash/zip-0212
    ///
    /// # Panics
    ///
    /// Panics if `plaintext` is shorter than [`COMPACT_NOTE_SIZE`].
    fn parse_note_plaintext_ivk(
        ivk: &PreparedIncomingViewingKey,
        plaintext: &[u8],
    ) -> Option<Note> {
        orchard_parse_note_plaintext(plaintext, |diversifier| {
            Some(DiversifiedTransmissionKey::derive(ivk, diversifier))
        })
    }

    /// Parses the given note plaintext from the sender's perspective.
    ///
    /// The implementation of this method must check that:
    /// - The note plaintext version is valid (for the given decryption domain's context,
    ///   which may be passed via `self`).
    /// - The note plaintext contains valid encodings of its various fields.
    /// - Any domain-specific requirements are satisfied.
    /// - `ephemeral_key` can be derived from `esk` and the diversifier within the note
    ///   plaintext.
    ///
    /// `&self` is passed here to enable the implementation to enforce contextual checks,
    /// such as rules like [ZIP 212] that become active at a specific block height.
    ///
    /// [ZIP 212]: https://zips.z.cash/zip-0212
    fn parse_note_plaintext_ovk(
        pk_d: &DiversifiedTransmissionKey,
        esk: &EphemeralSecretKey,
        ephemeral_key: &EphemeralKeyBytes,
        plaintext: &NotePlaintextBytes,
    ) -> Option<Note> {
        orchard_parse_note_plaintext(&plaintext.0, |diversifier| {
            if esk
                .derive_public(diversify_hash(diversifier.as_array()))
                .to_bytes()
                .0
                == ephemeral_key.0
            {
                Some(*pk_d)
            } else {
                None
            }
        })
    }

    /// Parses the `DiversifiedTransmissionKey` field of the outgoing plaintext.
    ///
    /// Returns `None` if `out_plaintext` does not contain a valid byte encoding of a
    /// `DiversifiedTransmissionKey`.
    fn extract_pk_d(out_plaintext: &OutPlaintextBytes) -> Option<DiversifiedTransmissionKey> {
        DiversifiedTransmissionKey::from_bytes(out_plaintext.0[0..32].try_into().unwrap()).into()
    }

    /// Parses the `EphemeralSecretKey` field of the outgoing plaintext.
    ///
    /// Returns `None` if `out_plaintext` does not contain a valid byte encoding of an
    /// `EphemeralSecretKey`.
    fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<EphemeralSecretKey> {
        EphemeralSecretKey::from_bytes(out_plaintext.0[32..OUT_PLAINTEXT_SIZE].try_into().unwrap())
            .into()
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;
    use super::{
        try_note_decryption, try_output_recovery_with_ovk,
        EphemeralKeyBytes
    };

    use super::{OrchardDomain, NoteEncryption};
    use crate::{
        keys::{
            DiversifiedTransmissionKey, Diversifier, EphemeralSecretKey, IncomingViewingKey,
//<<<<<<< HEAD
            OutgoingViewingKey, PreparedIncomingViewingKey, SpendingKey, FullViewingKey, Scope::External
//=======
//            OutgoingViewingKey, PreparedIncomingViewingKey,
//>>>>>>> d05b6cee9df7c4019509e2f54899b5979fb641b5
        },
        note::{NT_FT, Nullifier, RandomSeed, TransmittedNoteCiphertext},
        value::{NoteValue},
        Address, Note
    };

    #[test]
    fn test_vectors() {
        let test_vectors = crate::test_vectors::note_encryption::test_vectors();

        for tv in test_vectors {
            //
            // Load the test vector components
            //

            // Recipient key material
            let ivk = PreparedIncomingViewingKey::new(
                &IncomingViewingKey::from_bytes(&tv.incoming_viewing_key).unwrap(),
            );
            let ovk = OutgoingViewingKey::from(tv.ovk);
            let d = Diversifier::from_bytes(tv.default_d);
            let pk_d = DiversifiedTransmissionKey::from_bytes(&tv.default_pk_d).unwrap();

            // Received Action
            //let cv_net = ValueCommitment::from_bytes(&tv.cv_net).unwrap();
            let rho = Nullifier::from_bytes(&tv.rho).unwrap();
            //let cmx = ExtractedNoteCommitment::from_bytes(&tv.cmx).unwrap();

            let esk = EphemeralSecretKey::from_bytes(&tv.esk).unwrap();
            let ephemeral_key = EphemeralKeyBytes(tv.ephemeral_key);

            // Details about the expected note
            let d1 = NoteValue::from_raw(tv.v);
            let d2 = NoteValue::from_raw(tv.v);
            let sc = NoteValue::from_raw(tv.v);
            let nft = NoteValue::from_raw(0);
            let rseed = RandomSeed::from_bytes(tv.rseed, &rho).unwrap();
            let recipient = Address::from_parts(d, pk_d);
            let note = Note::from_parts(NT_FT, recipient, d1, d2, sc, nft, rho, rseed, tv.memo).unwrap();
            //let cmx = ExtractedNoteCommitment::from(note.commitment());

            //
            // Test the individual components
            //

            let shared_secret = esk.agree(&pk_d);
            assert_eq!(shared_secret.to_bytes(), tv.shared_secret);

            let k_enc = shared_secret.kdf_orchard(&ephemeral_key);
            assert_eq!(k_enc.as_bytes(), tv.k_enc);

            //let ock = prf_ock_orchard(&ovk, &ephemeral_key);
            //assert_eq!(ock.as_ref(), tv.ock);

//<<<<<<< HEAD
            let note_enc = NoteEncryption::new(Some(ovk.clone()), note);
            let mut rng = OsRng.clone();
//=======
//            let recipient = Address::from_parts(d, pk_d);
//            let note = Note::from_parts(recipient, value, rho, rseed).unwrap();
//            assert_eq!(ExtractedNoteCommitment::from(note.commitment()), cmx);
//>>>>>>> d05b6cee9df7c4019509e2f54899b5979fb641b5

            let encrypted_note = TransmittedNoteCiphertext {
                epk_bytes: ephemeral_key.0,
                enc_ciphertext: note_enc.encrypt_note_plaintext(),
                out_ciphertext: note_enc.encrypt_outgoing_plaintext(&mut rng),
            };

            //
            // Test decryption
            // (Tested first because it only requires immutable references.)
            //

            match try_note_decryption(&ivk, &encrypted_note) {
                Some(decrypted_note) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(&decrypted_note.memo()[..], &tv.memo[..]);
                }
                None => panic!("Note decryption failed"),
            }
            match try_output_recovery_with_ovk(&ovk, &encrypted_note) {
                Some(decrypted_note) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(&decrypted_note.memo()[..], &tv.memo[..]);
                }
                None => panic!("Output recovery failed"),
            }
        }
    }

    #[test]
    fn test_key_derivation_and_encryption()
    {
        let mut rng = OsRng.clone();

        // Alice' key material
        let sk_alice = SpendingKey::from_zip32_seed("This is Alice seed string! Usually this is just a listing of words. Here we just use sentences.".as_bytes(), 0, 0).unwrap();
        let fvk_alice = FullViewingKey::from(&sk_alice);

        // Bob's key material
        let sk_bob = SpendingKey::from_zip32_seed("This is Bob's seed string. His seed is a little shorter...".as_bytes(), 0, 0).unwrap();
        let fvk_bob = FullViewingKey::from(&sk_bob);
        let recipient = fvk_bob.address_at(0u32, External);

        // Note material
        let rho = Nullifier::from_bytes(&[1; 32]).unwrap();
        let note = Note::new(
            NT_FT,
            recipient,
            NoteValue::from_raw(100000),
            NoteValue::from_raw(357812230660),
            NoteValue::from_raw(123456789),
            NoteValue::from_raw(0),
            rho,
            rng,
            [0; 512]
        );

        // the ephermeral key pair which is used for encryption/decryption is derived deterministically from the note
        let esk = OrchardDomain::derive_esk(&note).unwrap();
        let epk = OrchardDomain::ka_derive_public(&note, &esk);
        
        let ne = NoteEncryption::new(Some(fvk_alice.to_ovk(External)), note);
        // a dummy action to test encryption/decryption
        let encrypted_note = TransmittedNoteCiphertext {
            epk_bytes: epk.to_bytes().0,
            enc_ciphertext: ne.encrypt_note_plaintext(),
            out_ciphertext: ne.encrypt_outgoing_plaintext(&mut rng),
        };

        // test receiver decryption
        match try_note_decryption(&PreparedIncomingViewingKey::new(&fvk_bob.to_ivk(External)), &encrypted_note) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note),
            None => panic!("Note decryption failed"),
        }

        // test sender decryption
        match try_output_recovery_with_ovk(&fvk_alice.to_ovk(External), &encrypted_note) {
            Some(decrypted_note) => assert_eq!(decrypted_note, note),
            None => panic!("Output recovery failed"),
        }

    }
}
