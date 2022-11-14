//! Data structures used for note construction.
use core::fmt;

use group::GroupEncoding;
use pasta_curves::pallas;
use rand::RngCore;
use subtle::CtOption;

use crate::{
    keys::{EphemeralSecretKey, FullViewingKey, Scope, SpendingKey},
    spec::{to_base, to_scalar, NonZeroPallasScalar, PrfExpand},
    value::NoteValue,
    Address,
    note_encryption::ENC_CIPHERTEXT_SIZE
};

pub(crate) mod commitment;
pub use self::commitment::{ExtractedNoteCommitment, NoteCommitment};

pub(crate) mod nullifier;
pub use self::nullifier::Nullifier;

/// The ZIP 212 seed randomness for a note.
#[derive(Copy, Clone, Debug)]
pub struct RandomSeed([u8; 32]);

impl RandomSeed {
    /// new random seed
    pub fn random(rng: &mut impl RngCore, rho: &Nullifier) -> Self {
        loop {
            let mut bytes = [0; 32];
            rng.fill_bytes(&mut bytes);
            let rseed = RandomSeed::from_bytes(bytes, rho);
            if rseed.is_some().into() {
                break rseed.unwrap();
            }
        }
    }

    /// from bytes
    pub fn from_bytes(rseed: [u8; 32], rho: &Nullifier) -> CtOption<Self> {
        let rseed = RandomSeed(rseed);
        let esk = rseed.esk_inner(rho);
        CtOption::new(rseed, esk.is_some())
    }

    /// as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    pub fn psi(&self, rho: &Nullifier) -> pallas::Base {
        to_base(PrfExpand::Psi.with_ad(&self.0, &rho.to_bytes()[..]))
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn esk_inner(&self, rho: &Nullifier) -> CtOption<NonZeroPallasScalar> {
        NonZeroPallasScalar::from_scalar(to_scalar(
            PrfExpand::Esk.with_ad(&self.0, &rho.to_bytes()[..]),
        ))
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn esk(&self, rho: &Nullifier) -> NonZeroPallasScalar {
        // We can't construct a RandomSeed for which this unwrap fails.
        self.esk_inner(rho).unwrap()
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    pub fn rcm(&self, rho: &Nullifier) -> commitment::NoteCommitTrapdoor {
        commitment::NoteCommitTrapdoor(to_scalar(
            PrfExpand::Rcm.with_ad(&self.0, &rho.to_bytes()[..]),
        ))
    }
}

/// The note types: fungible, non-fungible and auth
pub const NT_FT: u64    = 0x0;
pub const NT_NFT: u64   = 0x1;
pub const NT_AT: u64    = 0x2;

/// A discrete amount of funds received by an address.
#[derive(Debug, Copy, Clone)]
pub struct Note {
    /// The Header field with meta information like type of note.
    header: u64,
    /// The recipient of the funds.
    recipient: Address,
    /// The value of this note.
    d1: NoteValue,
    d2: NoteValue,
    sc: NoteValue,
    nft: NoteValue,
    /// A unique creation ID for this note.
    ///
    /// This is set to the nullifier of the note that was spent in the [`Action`] that
    /// created this note.
    ///
    /// [`Action`]: crate::action::Action
    rho: Nullifier,
    /// The seed randomness for various note components.
    rseed: RandomSeed,
    /// The 512 bytes memo field
    memo: [u8; 512],
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        // Notes are canonically defined by their commitments.
        ExtractedNoteCommitment::from(self.commitment())
            .eq(&ExtractedNoteCommitment::from(other.commitment()))
    }
}

impl Eq for Note {}

impl Note {
    pub(crate) fn from_parts(
        header: u64,
        recipient: Address,
        d1: NoteValue,
        d2: NoteValue,
        sc: NoteValue,
        nft: NoteValue,
        rho: Nullifier,
        rseed: RandomSeed,
        memo: [u8; 512]
    ) -> Self {
        Note {
            header,
            recipient,
            d1,
            d2,
            sc,
            nft,
            rho,
            rseed,
            memo
        }
    }

    /// Generates a new note.
    ///
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    pub fn new(
        header: u64,
        recipient: Address,
        d1: NoteValue,
        d2: NoteValue,
        sc: NoteValue,
        nft: NoteValue,
        rho: Nullifier,
        mut rng: impl RngCore,
        memo: [u8; 512]
    ) -> Self {
        loop {
            let note = Note {
                header,
                recipient,
                d1,
                d2,
                sc,
                nft,
                rho,
                rseed: RandomSeed::random(&mut rng, &rho),
                memo
            };
            if note.commitment_inner().is_some().into() {
                break note;
            }
        }
    }

    /// Generates a dummy spent note.
    ///
    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    pub fn dummy(
        rng: &mut impl RngCore,
        rho: Option<Nullifier>,
        val: Option<NoteValue>,
    ) -> (SpendingKey, FullViewingKey, Self) {
        let sk = SpendingKey::random(rng);
        let fvk: FullViewingKey = (&sk).into();
        let recipient = fvk.address_at(0u32, Scope::External);

        let note = Note::new(
            0,
            recipient,
            val.unwrap_or_else(|| NoteValue::zero()),
            NoteValue::zero(),
            NoteValue::zero(),
            NoteValue::zero(),
            rho.unwrap_or_else(|| Nullifier::dummy(rng)),
            rng,
            [0; 512]
        );

        (sk, fvk, note)
    }

    /// Returns the header field of this note.
    pub fn header(&self) -> u64 {
        self.header
    }

    /// Returns the recipient of this note.
    pub fn recipient(&self) -> Address {
        self.recipient
    }

    /// Returns the value of this note.
    pub fn d1(&self) -> NoteValue {
        self.d1
    }

    /// Returns the value of this note.
    pub fn d2(&self) -> NoteValue {
        self.d2
    }

    /// Returns the value of this note.
    pub fn sc(&self) -> NoteValue {
        self.sc
    }

    /// Returns the value of this note.
    pub fn nft(&self) -> NoteValue {
        self.nft
    }

    /// Returns the rseed value of this note.
    pub fn rseed(&self) -> &RandomSeed {
        &self.rseed
    }

    /// Derives the ephemeral secret key for this note.
    pub fn esk(&self) -> EphemeralSecretKey {
        EphemeralSecretKey(self.rseed.esk(&self.rho))
    }

    /// Returns rho of this note.
    pub fn rho(&self) -> Nullifier {
        self.rho
    }

    /// Returns memo of this note.
    pub fn memo(&self) -> [u8; 512] {
        self.memo
    }

    /// Derives the commitment to this note.
    ///
    /// Defined in [Zcash Protocol Spec § 3.2: Notes][notes].
    ///
    /// [notes]: https://zips.z.cash/protocol/nu5.pdf#notes
    pub fn commitment(&self) -> NoteCommitment {
        // `Note` will always have a note commitment by construction.
        self.commitment_inner().unwrap()
    }

    /// Derives the commitment to this note.
    ///
    /// This is the internal fallible API, used to check at construction time that the
    /// note has a commitment. Once you have a [`Note`] object, use `note.commitment()`
    /// instead.
    ///
    /// Defined in [Zcash Protocol Spec § 3.2: Notes][notes].
    ///
    /// [notes]: https://zips.z.cash/protocol/nu5.pdf#notes
    fn commitment_inner(&self) -> CtOption<NoteCommitment> {
        let g_d = self.recipient.g_d();

        NoteCommitment::derive(
            g_d.to_bytes(),
            self.recipient.pk_d().to_bytes(),
            self.d1,
            self.d2,
            self.sc,
            self.nft,
            self.rho.0,
            self.rseed.psi(&self.rho),
            self.rseed.rcm(&self.rho),
        )
    }

    /// Derives the nullifier for this note.
    pub fn nullifier(&self, fvk: &FullViewingKey) -> Nullifier {
        Nullifier::derive(
            fvk.nk(),
            self.rho.0,
            self.rseed.psi(&self.rho),
            self.commitment(),
        )
    }
}

/// An encrypted note.
#[derive(Clone)]
pub struct TransmittedNoteCiphertext {
    /// The serialization of the ephemeral public key
    pub epk_bytes: [u8; 32],
    /// The encrypted note ciphertext
    pub enc_ciphertext: [u8; ENC_CIPHERTEXT_SIZE],
    /// An encrypted value that allows the holder of the outgoing cipher
    /// key for the note to recover the note plaintext.
    pub out_ciphertext: [u8; 80],
}

impl fmt::Debug for TransmittedNoteCiphertext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransmittedNoteCiphertext")
            .field("epk_bytes", &self.epk_bytes)
            .field("enc_ciphertext", &hex::encode(self.enc_ciphertext))
            .field("out_ciphertext", &hex::encode(self.out_ciphertext))
            .finish()
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use proptest::prelude::*;

    use crate::{
        address::testing::arb_address, note::nullifier::testing::arb_nullifier, value::NoteValue,
    };

    use super::{Note, RandomSeed};

    prop_compose! {
        /// Generate an arbitrary random seed
        pub(crate) fn arb_rseed()(elems in prop::array::uniform32(prop::num::u8::ANY)) -> RandomSeed {
            RandomSeed(elems)
        }
    }

    prop_compose! {
        /// Generate an action without authorization data.
        pub fn arb_note(d1: NoteValue)(
            recipient in arb_address(),
            rho in arb_nullifier(),
            rseed in arb_rseed(),
        ) -> Note {
            Note {
                header: 0,
                recipient,
                d1,
                d2: d1,
                sc: d1,
                nft: NoteValue::from_raw(0),
                rho,
                rseed,
                memo: [0; 512]
            }
        }
    }
}
