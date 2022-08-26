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

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::convert::TryInto;

use chacha20::{
    cipher::{NewCipher, StreamCipher, StreamCipherSeek},
    ChaCha20,
};
use chacha20poly1305::{
    aead::{AeadInPlace, NewAead},
    ChaCha20Poly1305,
};

use rand_core::RngCore;
use subtle::{Choice, ConstantTimeEq};

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod batch
{
    //! APIs for batch trial decryption.

    use super::alloc::vec::Vec; // module is alloc only
    use core::iter;

    use super::{
        try_compact_note_decryption_inner, try_note_decryption_inner, BatchDomain, EphemeralKeyBytes,
        ShieldedOutput, COMPACT_NOTE_SIZE, ENC_CIPHERTEXT_SIZE,
    };

    /// Trial decryption of a batch of notes with a set of recipients.
    ///
    /// This is the batched version of [`crate::try_note_decryption`].
    #[allow(clippy::type_complexity)]
    pub fn try_note_decryption<D: BatchDomain, Output: ShieldedOutput<D, ENC_CIPHERTEXT_SIZE>>(
        ivks: &[D::IncomingViewingKey],
        outputs: &[(D, Output)],
    ) -> Vec<Option<(D::Note, D::Recipient, D::Memo)>> {
        batch_note_decryption(ivks, outputs, try_note_decryption_inner)
    }

    /// Trial decryption of a batch of notes for light clients with a set of recipients.
    ///
    /// This is the batched version of [`crate::try_compact_note_decryption`].
    pub fn try_compact_note_decryption<D: BatchDomain, Output: ShieldedOutput<D, COMPACT_NOTE_SIZE>>(
        ivks: &[D::IncomingViewingKey],
        outputs: &[(D, Output)],
    ) -> Vec<Option<(D::Note, D::Recipient)>> {
        batch_note_decryption(ivks, outputs, try_compact_note_decryption_inner)
    }

    fn batch_note_decryption<D: BatchDomain, Output: ShieldedOutput<D, CS>, F, FR, const CS: usize>(
        ivks: &[D::IncomingViewingKey],
        outputs: &[(D, Output)],
        decrypt_inner: F,
    ) -> Vec<Option<FR>>
    where
        F: Fn(&D, &D::IncomingViewingKey, &EphemeralKeyBytes, &Output, D::SymmetricKey) -> Option<FR>,
    {
        // Fetch the ephemeral keys for each output and batch-parse them.
        let ephemeral_keys = D::batch_epk(outputs.iter().map(|(_, output)| output.ephemeral_key()));

        // Derive the shared secrets for all combinations of (ivk, output).
        // The scalar multiplications cannot benefit from batching.
        let items = ivks.iter().flat_map(|ivk| {
            ephemeral_keys.iter().map(move |(epk, ephemeral_key)| {
                (
                    epk.as_ref().map(|epk| D::ka_agree_dec(ivk, epk)),
                    ephemeral_key,
                )
            })
        });

        // Run the batch-KDF to obtain the symmetric keys from the shared secrets.
        let keys = D::batch_kdf(items);

        // Finish the trial decryption!
        ivks.iter()
            .flat_map(|ivk| {
                // Reconstruct the matrix of (ivk, output) combinations.
                iter::repeat(ivk)
                    .zip(ephemeral_keys.iter())
                    .zip(outputs.iter())
            })
            .zip(keys)
            .map(|(((ivk, (_, ephemeral_key)), (domain, output)), key)| {
                // The `and_then` propagates any potential rejection from `D::epk`.
                key.and_then(|key| decrypt_inner(domain, ivk, ephemeral_key, output, key))
            })
            .collect()
    }
}

/// The size of a compact note.
pub const COMPACT_NOTE_SIZE: usize = 1 + // version
    11 + // diversifier
    8  + // d1
    8  + // d2
    8  + // sc
    8  + // nft
    32; // rseed (or rcm prior to ZIP 212)
/// The size of [`NotePlaintextBytes`].
pub const NOTE_PLAINTEXT_SIZE: usize = COMPACT_NOTE_SIZE + 512;
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

/// Trait that encapsulates protocol-specific note encryption types and logic.
///
/// This trait enables most of the note encryption logic to be shared between Sapling and
/// Orchard, as well as between different implementations of those protocols.
pub trait Domain {
    type EphemeralSecretKey: ConstantTimeEq;
    type EphemeralPublicKey;
    type SharedSecret;
    type SymmetricKey: AsRef<[u8]>;
    type Note;
    type Recipient;
    type DiversifiedTransmissionKey;
    type IncomingViewingKey;
    type OutgoingViewingKey;
    type ValueCommitment;
    type ExtractedCommitment;
    type ExtractedCommitmentBytes: Eq + for<'a> From<&'a Self::ExtractedCommitment>;
    type Memo;

    /// Derives the `EphemeralSecretKey` corresponding to this note.
    ///
    /// Returns `None` if the note was created prior to [ZIP 212], and doesn't have a
    /// deterministic `EphemeralSecretKey`.
    ///
    /// [ZIP 212]: https://zips.z.cash/zip-0212
    fn derive_esk(note: &Self::Note) -> Option<Self::EphemeralSecretKey>;

    /// Extracts the `DiversifiedTransmissionKey` from the note.
    fn get_pk_d(note: &Self::Note) -> Self::DiversifiedTransmissionKey;

    /// Derives `EphemeralPublicKey` from `esk` and the note's diversifier.
    fn ka_derive_public(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> Self::EphemeralPublicKey;

    /// Derives the `SharedSecret` from the sender's information during note encryption.
    fn ka_agree_enc(
        esk: &Self::EphemeralSecretKey,
        pk_d: &Self::DiversifiedTransmissionKey,
    ) -> Self::SharedSecret;

    /// Derives the `SharedSecret` from the recipient's information during note trial
    /// decryption.
    fn ka_agree_dec(
        ivk: &Self::IncomingViewingKey,
        epk: &Self::EphemeralPublicKey,
    ) -> Self::SharedSecret;

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
    fn kdf(secret: Self::SharedSecret, ephemeral_key: &EphemeralKeyBytes) -> Self::SymmetricKey;

    /// Encodes the given `Note` and `Memo` as a note plaintext.
    ///
    /// # Future breaking changes
    ///
    /// The `recipient` argument is present as a secondary way to obtain the diversifier;
    /// this is due to a historical quirk of how the Sapling `Note` struct was implemented
    /// in the `zcash_primitives` crate. `recipient` will be removed from this method in a
    /// future crate release, once [`zcash_primitives` has been refactored].
    ///
    /// [`zcash_primitives` has been refactored]: https://github.com/zcash/librustzcash/issues/454
    fn note_plaintext_bytes(
        note: &Self::Note,
        recipient: &Self::Recipient,
        memo: &Self::Memo,
    ) -> NotePlaintextBytes;

    /// Derives the [`OutgoingCipherKey`] for an encrypted note, given the note-specific
    /// public data and an `OutgoingViewingKey`.
    fn derive_ock(
        ovk: &Self::OutgoingViewingKey,
        //cv: &Self::ValueCommitment,
        cmstar_bytes: &Self::ExtractedCommitmentBytes,
        ephemeral_key: &EphemeralKeyBytes,
    ) -> OutgoingCipherKey;

    /// Encodes the outgoing plaintext for the given note.
    fn outgoing_plaintext_bytes(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> OutPlaintextBytes;

    /// Returns the byte encoding of the given `EphemeralPublicKey`.
    fn epk_bytes(epk: &Self::EphemeralPublicKey) -> EphemeralKeyBytes;

    /// Attempts to parse `ephemeral_key` as an `EphemeralPublicKey`.
    ///
    /// Returns `None` if `ephemeral_key` is not a valid byte encoding of an
    /// `EphemeralPublicKey`.
    fn epk(ephemeral_key: &EphemeralKeyBytes) -> Option<Self::EphemeralPublicKey>;

    /// Derives the `ExtractedCommitment` for this note.
    fn cmstar(note: &Self::Note) -> Self::ExtractedCommitment;

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
    fn parse_note_plaintext_without_memo_ivk(
        &self,
        ivk: &Self::IncomingViewingKey,
        plaintext: &[u8],
    ) -> Option<(Self::Note, Self::Recipient)>;

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
    fn parse_note_plaintext_without_memo_ovk(
        &self,
        pk_d: &Self::DiversifiedTransmissionKey,
        esk: &Self::EphemeralSecretKey,
        ephemeral_key: &EphemeralKeyBytes,
        plaintext: &NotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)>;

    /// Extracts the memo field from the given note plaintext.
    ///
    /// # Compatibility
    ///
    /// `&self` is passed here in anticipation of future changes to memo handling, where
    /// the memos may no longer be part of the note plaintext.
    fn extract_memo(&self, plaintext: &NotePlaintextBytes) -> Self::Memo;

    /// Parses the `DiversifiedTransmissionKey` field of the outgoing plaintext.
    ///
    /// Returns `None` if `out_plaintext` does not contain a valid byte encoding of a
    /// `DiversifiedTransmissionKey`.
    fn extract_pk_d(out_plaintext: &OutPlaintextBytes) -> Option<Self::DiversifiedTransmissionKey>;

    /// Parses the `EphemeralSecretKey` field of the outgoing plaintext.
    ///
    /// Returns `None` if `out_plaintext` does not contain a valid byte encoding of an
    /// `EphemeralSecretKey`.
    fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<Self::EphemeralSecretKey>;
}

/// Trait that encapsulates protocol-specific batch trial decryption logic.
///
/// Each batchable operation has a default implementation that calls through to the
/// non-batched implementation. Domains can override whichever operations benefit from
/// batched logic.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub trait BatchDomain: Domain {
    /// Computes `Self::kdf` on a batch of items.
    ///
    /// For each item in the batch, if the shared secret is `None`, this returns `None` at
    /// that position.
    fn batch_kdf<'a>(
        items: impl Iterator<Item = (Option<Self::SharedSecret>, &'a EphemeralKeyBytes)>,
    ) -> Vec<Option<Self::SymmetricKey>> {
        // Default implementation: do the non-batched thing.
        items
            .map(|(secret, ephemeral_key)| secret.map(|secret| Self::kdf(secret, ephemeral_key)))
            .collect()
    }

    /// Computes `Self::epk` on a batch of ephemeral keys.
    ///
    /// This is useful for protocols where the underlying curve requires an inversion to
    /// parse an encoded point.
    ///
    /// For usability, this returns tuples of the ephemeral keys and the result of parsing
    /// them.
    fn batch_epk(
        ephemeral_keys: impl Iterator<Item = EphemeralKeyBytes>,
    ) -> Vec<(Option<Self::EphemeralPublicKey>, EphemeralKeyBytes)> {
        // Default implementation: do the non-batched thing.
        ephemeral_keys
            .map(|ephemeral_key| (Self::epk(&ephemeral_key), ephemeral_key))
            .collect()
    }
}

/// Trait that provides access to the components of an encrypted transaction output.
///
/// Implementations of this trait are required to define the length of their ciphertext
/// field. In order to use the trial decryption APIs in this crate, the length must be
/// either [`ENC_CIPHERTEXT_SIZE`] or [`COMPACT_NOTE_SIZE`].
pub trait ShieldedOutput<D: Domain, const CIPHERTEXT_SIZE: usize> {
    /// Exposes the `ephemeral_key` field of the output.
    fn ephemeral_key(&self) -> EphemeralKeyBytes;

    /// Exposes the `cmu_bytes` or `cmx_bytes` field of the output.
    fn cmstar_bytes(&self) -> D::ExtractedCommitmentBytes;

    /// Exposes the note ciphertext of the output.
    fn enc_ciphertext(&self) -> &[u8; CIPHERTEXT_SIZE];
}

/// A struct containing context required for encrypting Sapling and Orchard notes.
///
/// This struct provides a safe API for encrypting Sapling and Orchard notes. In particular, it
/// enforces that fresh ephemeral keys are used for every note, and that the ciphertexts are
/// consistent with each other.
///
/// Implements section 4.19 of the
/// [Zcash Protocol Specification](https://zips.z.cash/protocol/nu5.pdf#saplingandorchardinband)
/// NB: the example code is only covering the post-Canopy case.
///
/// # Examples
///
/// ```
/// extern crate ff;
/// extern crate rand_core;
/// extern crate zcash_primitives;
///
/// use ff::Field;
/// use rand_core::OsRng;
/// use zcash_primitives::{
///     consensus::{TEST_NETWORK, TestNetwork, NetworkUpgrade, Parameters},
///     memo::MemoBytes,
///     sapling::{
///         keys::{OutgoingViewingKey, prf_expand},
///         note_encryption::sapling_note_encryption,
///         util::generate_random_rseed,
///         Diversifier, PaymentAddress, Rseed, ValueCommitment
///     },
/// };
///
/// let mut rng = OsRng;
///
/// let diversifier = Diversifier([0; 11]);
/// let pk_d = diversifier.g_d().unwrap();
/// let to = PaymentAddress::from_parts(diversifier, pk_d).unwrap();
/// let ovk = Some(OutgoingViewingKey([0; 32]));
///
/// let value = 1000;
/// let rcv = jubjub::Fr::random(&mut rng);
/// let cv = ValueCommitment {
///     value,
///     randomness: rcv.clone(),
/// };
/// let height = TEST_NETWORK.activation_height(NetworkUpgrade::Canopy).unwrap();
/// let rseed = generate_random_rseed(&TEST_NETWORK, height, &mut rng);
/// let note = to.create_note(value, rseed).unwrap();
/// let cmu = note.cmu();
///
/// let mut enc = sapling_note_encryption::<_, TestNetwork>(ovk, note, to, MemoBytes::empty(), &mut rng);
/// let encCiphertext = enc.encrypt_note_plaintext();
/// let outCiphertext = enc.encrypt_outgoing_plaintext(&cv.commitment().into(), &cmu, &mut rng);
/// ```
#[derive(Debug)]
pub struct NoteEncryption<D: Domain> {
    epk: D::EphemeralPublicKey,
    esk: D::EphemeralSecretKey,
    note: D::Note,
    to: D::Recipient,
    memo: D::Memo,
    /// `None` represents the `ovk = ⊥` case.
    ovk: Option<D::OutgoingViewingKey>,
}

impl<D: Domain> NoteEncryption<D> {
    /// Construct a new note encryption context for the specified note,
    /// recipient, and memo.
    pub fn new(
        ovk: Option<D::OutgoingViewingKey>,
        note: D::Note,
        to: D::Recipient,
        memo: D::Memo,
    ) -> Self {
        let esk = D::derive_esk(&note).expect("ZIP 212 is active.");
        NoteEncryption {
            epk: D::ka_derive_public(&note, &esk),
            esk,
            note,
            to,
            memo,
            ovk,
        }
    }

    /// For use only with Sapling. This method is preserved in order that test code
    /// be able to generate pre-ZIP-212 ciphertexts so that tests can continue to
    /// cover pre-ZIP-212 transaction decryption.
    #[cfg(feature = "pre-zip-212")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pre-zip-212")))]
    pub fn new_with_esk(
        esk: D::EphemeralSecretKey,
        ovk: Option<D::OutgoingViewingKey>,
        note: D::Note,
        to: D::Recipient,
        memo: D::Memo,
    ) -> Self {
        NoteEncryption {
            epk: D::ka_derive_public(&note, &esk),
            esk,
            note,
            to,
            memo,
            ovk,
        }
    }

    /// Exposes the ephemeral secret key being used to encrypt this note.
    pub fn esk(&self) -> &D::EphemeralSecretKey {
        &self.esk
    }

    /// Exposes the encoding of the ephemeral public key being used to encrypt this note.
    pub fn epk(&self) -> &D::EphemeralPublicKey {
        &self.epk
    }

    /// Generates `encCiphertext` for this note.
    pub fn encrypt_note_plaintext(&self) -> [u8; ENC_CIPHERTEXT_SIZE] {
        let pk_d = D::get_pk_d(&self.note);
        let shared_secret = D::ka_agree_enc(&self.esk, &pk_d);
        let key = D::kdf(shared_secret, &D::epk_bytes(&self.epk));
        let input = D::note_plaintext_bytes(&self.note, &self.to, &self.memo);

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
        //cv: &D::ValueCommitment,
        cmstar: &D::ExtractedCommitment,
        rng: &mut R,
    ) -> [u8; OUT_CIPHERTEXT_SIZE] {
        let (ock, input) = if let Some(ovk) = &self.ovk {
            let ock = D::derive_ock(ovk, /*&cv,*/ &cmstar.into(), &D::epk_bytes(&self.epk));
            let input = D::outgoing_plaintext_bytes(&self.note, &self.esk);

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
pub fn try_note_decryption<D: Domain, Output: ShieldedOutput<D, ENC_CIPHERTEXT_SIZE>>(
    domain: &D,
    ivk: &D::IncomingViewingKey,
    output: &Output,
) -> Option<(D::Note, D::Recipient, D::Memo)> {
    let ephemeral_key = output.ephemeral_key();

    let epk = D::epk(&ephemeral_key)?;
    let shared_secret = D::ka_agree_dec(ivk, &epk);
    let key = D::kdf(shared_secret, &ephemeral_key);

    try_note_decryption_inner(domain, ivk, &ephemeral_key, output, key)
}

fn try_note_decryption_inner<D: Domain, Output: ShieldedOutput<D, ENC_CIPHERTEXT_SIZE>>(
    domain: &D,
    ivk: &D::IncomingViewingKey,
    ephemeral_key: &EphemeralKeyBytes,
    output: &Output,
    key: D::SymmetricKey,
) -> Option<(D::Note, D::Recipient, D::Memo)> {
    let enc_ciphertext = output.enc_ciphertext();

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

    let (note, to) = parse_note_plaintext_without_memo_ivk(
        domain,
        ivk,
        ephemeral_key,
        &output.cmstar_bytes(),
        &plaintext.0,
    )?;
    let memo = domain.extract_memo(&plaintext);

    Some((note, to, memo))
}

fn parse_note_plaintext_without_memo_ivk<D: Domain>(
    domain: &D,
    ivk: &D::IncomingViewingKey,
    ephemeral_key: &EphemeralKeyBytes,
    cmstar_bytes: &D::ExtractedCommitmentBytes,
    plaintext: &[u8],
) -> Option<(D::Note, D::Recipient)> {
    let (note, to) = domain.parse_note_plaintext_without_memo_ivk(ivk, &plaintext)?;

    if let NoteValidity::Valid = check_note_validity::<D>(&note, ephemeral_key, cmstar_bytes) {
        Some((note, to))
    } else {
        None
    }
}

fn check_note_validity<D: Domain>(
    note: &D::Note,
    ephemeral_key: &EphemeralKeyBytes,
    cmstar_bytes: &D::ExtractedCommitmentBytes,
) -> NoteValidity {
    if &D::ExtractedCommitmentBytes::from(&D::cmstar(&note)) == cmstar_bytes {
        if let Some(derived_esk) = D::derive_esk(note) {
            if D::epk_bytes(&D::ka_derive_public(&note, &derived_esk))
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
    } else {
        // Published commitment doesn't match calculated commitment
        NoteValidity::Invalid
    }
}

/// Trial decryption of the compact note plaintext by the recipient for light clients.
///
/// Attempts to decrypt and validate the given compact shielded output using the
/// given `ivk`. If successful, the corresponding note is returned, along with the address
/// to which the note was sent.
///
/// Implements the procedure specified in [`ZIP 307`].
///
/// [`ZIP 307`]: https://zips.z.cash/zip-0307
pub fn try_compact_note_decryption<D: Domain, Output: ShieldedOutput<D, COMPACT_NOTE_SIZE>>(
    domain: &D,
    ivk: &D::IncomingViewingKey,
    output: &Output,
) -> Option<(D::Note, D::Recipient)> {
    let ephemeral_key = output.ephemeral_key();

    let epk = D::epk(&ephemeral_key)?;
    let shared_secret = D::ka_agree_dec(&ivk, &epk);
    let key = D::kdf(shared_secret, &ephemeral_key);

    try_compact_note_decryption_inner(domain, ivk, &ephemeral_key, output, key)
}

fn try_compact_note_decryption_inner<D: Domain, Output: ShieldedOutput<D, COMPACT_NOTE_SIZE>>(
    domain: &D,
    ivk: &D::IncomingViewingKey,
    ephemeral_key: &EphemeralKeyBytes,
    output: &Output,
    key: D::SymmetricKey,
) -> Option<(D::Note, D::Recipient)> {
    // Start from block 1 to skip over Poly1305 keying output
    let mut plaintext = [0; COMPACT_NOTE_SIZE];
    plaintext.copy_from_slice(output.enc_ciphertext());
    let mut keystream = ChaCha20::new(key.as_ref().into(), [0u8; 12][..].into());
    keystream.seek(64);
    keystream.apply_keystream(&mut plaintext);

    parse_note_plaintext_without_memo_ivk(
        domain,
        ivk,
        ephemeral_key,
        &output.cmstar_bytes(),
        &plaintext,
    )
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
pub fn try_output_recovery_with_ovk<D: Domain, Output: ShieldedOutput<D, ENC_CIPHERTEXT_SIZE>>(
    domain: &D,
    ovk: &D::OutgoingViewingKey,
    output: &Output,
    //cv: &D::ValueCommitment,
    out_ciphertext: &[u8; OUT_CIPHERTEXT_SIZE],
) -> Option<(D::Note, D::Recipient, D::Memo)> {
    let ock = D::derive_ock(ovk, /*&cv,*/ &output.cmstar_bytes(), &output.ephemeral_key());
    try_output_recovery_with_ock(domain, &ock, output, out_ciphertext)
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
pub fn try_output_recovery_with_ock<D: Domain, Output: ShieldedOutput<D, ENC_CIPHERTEXT_SIZE>>(
    domain: &D,
    ock: &OutgoingCipherKey,
    output: &Output,
    out_ciphertext: &[u8; OUT_CIPHERTEXT_SIZE],
) -> Option<(D::Note, D::Recipient, D::Memo)> {
    let enc_ciphertext = output.enc_ciphertext();

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

    let pk_d = D::extract_pk_d(&op)?;
    let esk = D::extract_esk(&op)?;

    let ephemeral_key = output.ephemeral_key();
    let shared_secret = D::ka_agree_enc(&esk, &pk_d);
    // The small-order point check at the point of output parsing rejects
    // non-canonical encodings, so reencoding here for the KDF should
    // be okay.
    let key = D::kdf(shared_secret, &ephemeral_key);

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

    let (note, to) =
        domain.parse_note_plaintext_without_memo_ovk(&pk_d, &esk, &ephemeral_key, &plaintext)?;
    let memo = domain.extract_memo(&plaintext);

    // ZIP 212: Check that the esk provided to this function is consistent with the esk we
    // can derive from the note.
    if let Some(derived_esk) = D::derive_esk(&note) {
        if (!derived_esk.ct_eq(&esk)).into() {
            return None;
        }
    }

    if let NoteValidity::Valid =
        check_note_validity::<D>(&note, &ephemeral_key, &output.cmstar_bytes())
    {
        Some((note, to, memo))
    } else {
        None
    }
}




// ! In-band secret distribution for Orchard bundles.

use core::fmt;

use blake2b_simd::{Hash, Params};
use group::ff::PrimeField;
//use zcash_note_encryption::{
//    BatchDomain, Domain, EphemeralKeyBytes, NotePlaintextBytes, OutPlaintextBytes,
//    OutgoingCipherKey, ShieldedOutput, COMPACT_NOTE_SIZE, ENC_CIPHERTEXT_SIZE, NOTE_PLAINTEXT_SIZE,
//    OUT_PLAINTEXT_SIZE,
//};

use crate::{
    action::Action,
    keys::{
        DiversifiedTransmissionKey, Diversifier, EphemeralPublicKey, EphemeralSecretKey,
        IncomingViewingKey, OutgoingViewingKey, SharedSecret,
    },
    note::{ExtractedNoteCommitment, Nullifier, RandomSeed},
    spec::diversify_hash,
    value::{NoteValue, ValueCommitment},
    Address, Note,
};

const PRF_OCK_ORCHARD_PERSONALIZATION: &[u8; 16] = b"Zcash_Orchardock";

/// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][concreteprfs].
///
/// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
pub(crate) fn prf_ock_orchard(
    ovk: &OutgoingViewingKey,
    //cv: &ValueCommitment,
    cmx_bytes: &[u8; 32],
    ephemeral_key: &EphemeralKeyBytes,
) -> OutgoingCipherKey {
    OutgoingCipherKey(
        Params::new()
            .hash_length(32)
            .personal(PRF_OCK_ORCHARD_PERSONALIZATION)
            .to_state()
            .update(ovk.as_ref())
            //.update(&cv.to_bytes())
            .update(cmx_bytes)
            .update(ephemeral_key.as_ref())
            .finalize()
            .as_bytes()
            .try_into()
            .unwrap(),
    )
}

fn orchard_parse_note_plaintext_without_memo<F>(
    domain: &OrchardDomain,
    plaintext: &[u8],
    get_validated_pk_d: F,
) -> Option<(Note, Address)>
where
    F: FnOnce(&Diversifier) -> Option<DiversifiedTransmissionKey>,
{
    assert!(plaintext.len() >= COMPACT_NOTE_SIZE);

    // Check note plaintext version
    if plaintext[0] != 0x02 {
        return None;
    }

    // The unwraps below are guaranteed to succeed by the assertion above
    let diversifier = Diversifier::from_bytes(plaintext[1..12].try_into().unwrap());
    let d1 = NoteValue::from_bytes(plaintext[12..20].try_into().unwrap());
    let d2 = NoteValue::from_bytes(plaintext[20..28].try_into().unwrap());
    let sc = NoteValue::from_bytes(plaintext[28..36].try_into().unwrap());
    let nft = NoteValue::from_bytes(plaintext[36..44].try_into().unwrap());
    let rseed = Option::from(RandomSeed::from_bytes(
        plaintext[44..COMPACT_NOTE_SIZE].try_into().unwrap(),
        &domain.rho,
    ))?;

    let pk_d = get_validated_pk_d(&diversifier)?;

    let recipient = Address::from_parts(diversifier, pk_d);
    let note = Note::from_parts(recipient, d1, d2, sc, nft, domain.rho, rseed);
    Some((note, recipient))
}

/// Orchard-specific note encryption logic.
#[derive(Debug)]
pub struct OrchardDomain {
    rho: Nullifier,
}

impl OrchardDomain {
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
    type SharedSecret = SharedSecret;
    type SymmetricKey = Hash;
    type Note = Note;
    type Recipient = Address;
    type DiversifiedTransmissionKey = DiversifiedTransmissionKey;
    type IncomingViewingKey = IncomingViewingKey;
    type OutgoingViewingKey = OutgoingViewingKey;
    type ValueCommitment = ValueCommitment;
    type ExtractedCommitment = ExtractedNoteCommitment;
    type ExtractedCommitmentBytes = [u8; 32];
    type Memo = [u8; 512]; // TODO use a more interesting type

    fn derive_esk(note: &Self::Note) -> Option<Self::EphemeralSecretKey> {
        Some(note.esk())
    }

    fn get_pk_d(note: &Self::Note) -> Self::DiversifiedTransmissionKey {
        *note.recipient().pk_d()
    }

    fn ka_derive_public(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> Self::EphemeralPublicKey {
        esk.derive_public(note.recipient().g_d())
    }

    fn ka_agree_enc(
        esk: &Self::EphemeralSecretKey,
        pk_d: &Self::DiversifiedTransmissionKey,
    ) -> Self::SharedSecret {
        esk.agree(pk_d)
    }

    fn ka_agree_dec(
        ivk: &Self::IncomingViewingKey,
        epk: &Self::EphemeralPublicKey,
    ) -> Self::SharedSecret {
        epk.agree(ivk)
    }

    fn kdf(secret: Self::SharedSecret, ephemeral_key: &EphemeralKeyBytes) -> Self::SymmetricKey {
        secret.kdf_orchard(ephemeral_key)
    }

    fn note_plaintext_bytes(
        note: &Self::Note,
        _: &Self::Recipient,
        memo: &Self::Memo,
    ) -> NotePlaintextBytes {
        let mut np = [0; NOTE_PLAINTEXT_SIZE];
        np[0] = 0x02;
        np[1..12].copy_from_slice(note.recipient().diversifier().as_array());
        np[12..20].copy_from_slice(&note.d1().to_bytes());
        np[20..28].copy_from_slice(&note.d2().to_bytes());
        np[28..36].copy_from_slice(&note.sc().to_bytes());
        np[36..44].copy_from_slice(&note.nft().to_bytes());
        np[44..76].copy_from_slice(note.rseed().as_bytes());
        np[76..].copy_from_slice(memo);
        NotePlaintextBytes(np)
    }

    fn derive_ock(
        ovk: &Self::OutgoingViewingKey,
        //cv: &Self::ValueCommitment,
        cmstar_bytes: &Self::ExtractedCommitmentBytes,
        ephemeral_key: &EphemeralKeyBytes,
    ) -> OutgoingCipherKey {
        prf_ock_orchard(ovk, /*cv,*/ cmstar_bytes, ephemeral_key)
    }

    fn outgoing_plaintext_bytes(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> OutPlaintextBytes {
        let mut op = [0; OUT_PLAINTEXT_SIZE];
        op[..32].copy_from_slice(&note.recipient().pk_d().to_bytes());
        op[32..].copy_from_slice(&esk.0.to_repr());
        OutPlaintextBytes(op)
    }

    fn epk_bytes(epk: &Self::EphemeralPublicKey) -> EphemeralKeyBytes {
        epk.to_bytes()
    }

    fn epk(ephemeral_key: &EphemeralKeyBytes) -> Option<Self::EphemeralPublicKey> {
        EphemeralPublicKey::from_bytes(&ephemeral_key.0).into()
    }

    fn cmstar(note: &Self::Note) -> Self::ExtractedCommitment {
        note.commitment().into()
    }

    fn parse_note_plaintext_without_memo_ivk(
        &self,
        ivk: &Self::IncomingViewingKey,
        plaintext: &[u8],
    ) -> Option<(Self::Note, Self::Recipient)> {
        orchard_parse_note_plaintext_without_memo(self, plaintext, |diversifier| {
            Some(DiversifiedTransmissionKey::derive(ivk, diversifier))
        })
    }

    fn parse_note_plaintext_without_memo_ovk(
        &self,
        pk_d: &Self::DiversifiedTransmissionKey,
        esk: &Self::EphemeralSecretKey,
        ephemeral_key: &EphemeralKeyBytes,
        plaintext: &NotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        orchard_parse_note_plaintext_without_memo(self, &plaintext.0, |diversifier| {
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

    fn extract_memo(&self, plaintext: &NotePlaintextBytes) -> Self::Memo {
        plaintext.0[COMPACT_NOTE_SIZE..NOTE_PLAINTEXT_SIZE]
            .try_into()
            .unwrap()
    }

    fn extract_pk_d(out_plaintext: &OutPlaintextBytes) -> Option<Self::DiversifiedTransmissionKey> {
        DiversifiedTransmissionKey::from_bytes(out_plaintext.0[0..32].try_into().unwrap()).into()
    }

    fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<Self::EphemeralSecretKey> {
        EphemeralSecretKey::from_bytes(out_plaintext.0[32..OUT_PLAINTEXT_SIZE].try_into().unwrap())
            .into()
    }
}

impl BatchDomain for OrchardDomain {
    fn batch_kdf<'a>(
        items: impl Iterator<Item = (Option<Self::SharedSecret>, &'a EphemeralKeyBytes)>,
    ) -> Vec<Option<Self::SymmetricKey>> {
        let (shared_secrets, ephemeral_keys): (Vec<_>, Vec<_>) = items.unzip();

        SharedSecret::batch_to_affine(shared_secrets)
            .zip(ephemeral_keys.into_iter())
            .map(|(secret, ephemeral_key)| {
                secret.map(|dhsecret| SharedSecret::kdf_orchard_inner(dhsecret, ephemeral_key))
            })
            .collect()
    }
}

/// Implementation of in-band secret distribution for Orchard bundles.
pub type OrchardNoteEncryption = NoteEncryption<OrchardDomain>;

impl<T> ShieldedOutput<OrchardDomain, ENC_CIPHERTEXT_SIZE> for Action<T> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.encrypted_note().epk_bytes)
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx().to_bytes()
    }

    fn enc_ciphertext(&self) -> &[u8; ENC_CIPHERTEXT_SIZE] {
        &self.encrypted_note().enc_ciphertext
    }
}

/// A compact Action for light clients.
pub struct CompactAction {
    nullifier: Nullifier,
    cmx: ExtractedNoteCommitment,
    ephemeral_key: EphemeralKeyBytes,
    enc_ciphertext: [u8; COMPACT_NOTE_SIZE],
}

impl fmt::Debug for CompactAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactAction")
    }
}

impl<T> From<&Action<T>> for CompactAction {
    fn from(action: &Action<T>) -> Self {
        CompactAction {
            nullifier: *action.nullifier(),
            cmx: *action.cmx(),
            ephemeral_key: action.ephemeral_key(),
            enc_ciphertext: action.encrypted_note().enc_ciphertext[..52]
                .try_into()
                .unwrap(),
        }
    }
}

impl ShieldedOutput<OrchardDomain, COMPACT_NOTE_SIZE> for CompactAction {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.ephemeral_key.0)
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx.to_bytes()
    }

    fn enc_ciphertext(&self) -> &[u8; COMPACT_NOTE_SIZE] {
        &self.enc_ciphertext
    }
}

impl CompactAction {
    /// Create a CompactAction from its constituent parts
    pub fn from_parts(
        nullifier: Nullifier,
        cmx: ExtractedNoteCommitment,
        ephemeral_key: EphemeralKeyBytes,
        enc_ciphertext: [u8; COMPACT_NOTE_SIZE],
    ) -> Self {
        Self {
            nullifier,
            cmx,
            ephemeral_key,
            enc_ciphertext,
        }
    }

    ///Returns the nullifier of the note being spent.
    pub fn nullifier(&self) -> Nullifier {
        self.nullifier
    }
}

#[cfg(test)]
mod tests {
    use group::GroupEncoding;
    use rand::rngs::OsRng;
    use super::{
        try_compact_note_decryption, try_note_decryption, try_output_recovery_with_ovk,
        EphemeralKeyBytes, NoteEncryption, Domain,
    };

    use super::{prf_ock_orchard, CompactAction, OrchardDomain, OrchardNoteEncryption};
    use crate::value::{ValueSum, ValueCommitTrapdoor};
    use crate::{
        action::Action,
        keys::{
            DiversifiedTransmissionKey, Diversifier, EphemeralSecretKey, IncomingViewingKey,
            OutgoingViewingKey, SpendingKey, FullViewingKey, Scope::External
        },
        note::{ExtractedNoteCommitment, Nullifier, RandomSeed, TransmittedNoteCiphertext},
        primitives::redpallas,
        value::{NoteValue, ValueCommitment},
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
            let ivk = IncomingViewingKey::from_bytes(&tv.incoming_viewing_key).unwrap();
            let ovk = OutgoingViewingKey::from(tv.ovk);
            let d = Diversifier::from_bytes(tv.default_d);
            let pk_d = DiversifiedTransmissionKey::from_bytes(&tv.default_pk_d).unwrap();

            // Received Action
            let cv_net = ValueCommitment::from_bytes(&tv.cv_net).unwrap();
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
            let note = Note::from_parts(recipient, d1, d2, sc, nft, rho, rseed);
            let cmx = ExtractedNoteCommitment::from(note.commitment());

            //
            // Test the individual components
            //

            let shared_secret = esk.agree(&pk_d);
            assert_eq!(shared_secret.to_bytes(), tv.shared_secret);

            let k_enc = shared_secret.kdf_orchard(&ephemeral_key);
            assert_eq!(k_enc.as_bytes(), tv.k_enc);

            let ock = prf_ock_orchard(&ovk, /*&cv_net,*/ &cmx.to_bytes(), &ephemeral_key);
            //assert_eq!(ock.as_ref(), tv.ock);

            let note_enc = OrchardNoteEncryption::new(Some(ovk.clone()), note, recipient, tv.memo);
            let mut rng = OsRng.clone();

            let action = Action::from_parts(
                // rho is the nullifier in the receiving Action.
                rho,
                // We don't need a valid rk for this test.
                redpallas::VerificationKey::dummy(),
                cmx,
                TransmittedNoteCiphertext {
                    epk_bytes: ephemeral_key.0,
                    enc_ciphertext: note_enc.encrypt_note_plaintext(),
                    out_ciphertext: note_enc.encrypt_outgoing_plaintext(&cmx, &mut rng),
                },
                cv_net.clone(),
                (),
            );

            //
            // Test decryption
            // (Tested first because it only requires immutable references.)
            //

            let domain = OrchardDomain { rho };

            match try_note_decryption(&domain, &ivk, &action) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Note decryption failed"),
            }
/*
            match try_compact_note_decryption(&domain, &ivk, &CompactAction::from(&action)) {
                Some((decrypted_note, decrypted_to)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                }
                None => panic!("Compact note decryption failed"),
            }
 */
            match try_output_recovery_with_ovk(&domain, &ovk, &action, /*&cv_net,*/ &note_enc.encrypt_outgoing_plaintext(&cmx, &mut rng)) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Output recovery failed"),
            }

            //
            // Test encryption
            //
/*
            let ne = OrchardNoteEncryption::new_with_esk(esk, Some(ovk), note, recipient, tv.memo);

            assert_eq!(ne.encrypt_note_plaintext().as_ref(), &tv.c_enc[..]);
            assert_eq!(
                &ne.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut OsRng)[..],
                &tv.c_out[..]
            );
 */
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
        let note = Note::new(recipient,
                                   NoteValue::from_raw(100000),
                                   NoteValue::from_raw(357812230660),
                                   NoteValue::from_raw(123456789),
                                   NoteValue::from_raw(0),
                                   rho,
                                   rng);
        let cmx = ExtractedNoteCommitment::from(note.commitment());

        // the ephermeral key pair which is used for encryption/decryption is derived deterministically from the note
        let esk = OrchardDomain::derive_esk(&note).unwrap();
        let epk = OrchardDomain::ka_derive_public(&note, &esk);
        
        let ne = OrchardNoteEncryption::new(Some(fvk_alice.to_ovk(External)),
                                                                           note, recipient, [0; 512]);
        // a dummy action to test encryption/decryption
        let action = Action::from_parts(
            rho,
            redpallas::VerificationKey::dummy(),
            cmx,
            TransmittedNoteCiphertext {
                epk_bytes: epk.to_bytes().0,
                enc_ciphertext: ne.encrypt_note_plaintext(),
                out_ciphertext: ne.encrypt_outgoing_plaintext(&cmx, &mut rng),
            },
            ValueCommitment::derive(ValueSum::from_raw(0), ValueCommitTrapdoor::random(rng)),
            (),
        );

        let domain = OrchardDomain { rho };

        // test receiver decryption
        match try_note_decryption(&domain, &fvk_bob.to_ivk(External), &action) {
            Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                assert_eq!(decrypted_note, note);
                assert_eq!(decrypted_to, recipient);
                assert_eq!(&decrypted_memo[..], &[0; 512]);
            }
            None => panic!("Note decryption failed"),
        }

        // test sender decryption
        match try_output_recovery_with_ovk(&domain, &fvk_alice.to_ovk(External), &action, &ne.encrypt_outgoing_plaintext(&cmx, &mut rng)) {
            Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                assert_eq!(decrypted_note, note);
                assert_eq!(decrypted_to, recipient);
                assert_eq!(&decrypted_memo[..], &[0; 512]);
            }
            None => panic!("Output recovery failed"),
        }

    }
}
