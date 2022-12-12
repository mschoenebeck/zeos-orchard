use subtle::CtOption;

use crate::{
    keys::{DiversifiedTransmissionKey, Diversifier},
    spec::{diversify_hash, NonIdentityPallasPoint},
};
use rand_core::RngCore;
use crate::FullViewingKey;
use crate::SpendingKey;
use crate::keys::Scope;
use bech32::{FromBase32, ToBase32, Variant};

/// A shielded payment address.
///
/// # Examples
///
/// ```
/// use orchard::keys::{SpendingKey, FullViewingKey, Scope};
///
/// let sk = SpendingKey::from_bytes([7; 32]).unwrap();
/// let address = FullViewingKey::from(&sk).address_at(0u32, Scope::External);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address {
    d: Diversifier,
    pk_d: DiversifiedTransmissionKey,
}

impl Address {
    pub(crate) fn from_parts(d: Diversifier, pk_d: DiversifiedTransmissionKey) -> Self {
        // We assume here that pk_d is correctly-derived from d. We ensure this for
        // internal APIs. For parsing from raw byte encodings, we assume that users aren't
        // modifying internals of encoded address formats. If they do, that can result in
        // lost funds, but we can't defend against that from here.
        Address { d, pk_d }
    }

    /// creates a dummy address
    pub fn dummy(rng: &mut impl RngCore) -> Self
    {
        let sk = SpendingKey::random(rng);
        let fvk: FullViewingKey = (&sk).into();
        fvk.address_at(0u32, Scope::External)
    }

    /// Returns the [`Diversifier`] for this `Address`.
    pub fn diversifier(&self) -> Diversifier {
        self.d
    }

    /// returns g_d
    pub fn g_d(&self) -> NonIdentityPallasPoint {
        diversify_hash(self.d.as_array())
    }

    /// returns pk_d
    pub fn pk_d(&self) -> &DiversifiedTransmissionKey {
        &self.pk_d
    }

    /// Serializes this address to its "raw" encoding as specified in [Zcash Protocol Spec § 5.6.4.2: Orchard Raw Payment Addresses][orchardpaymentaddrencoding]
    ///
    /// [orchardpaymentaddrencoding]: https://zips.z.cash/protocol/protocol.pdf#orchardpaymentaddrencoding
    pub fn to_raw_address_bytes(&self) -> [u8; 43] {
        let mut result = [0u8; 43];
        result[..11].copy_from_slice(self.d.as_array());
        result[11..].copy_from_slice(&self.pk_d.to_bytes());
        result
    }

    /// Parse an address from its "raw" encoding as specified in [Zcash Protocol Spec § 5.6.4.2: Orchard Raw Payment Addresses][orchardpaymentaddrencoding]
    ///
    /// [orchardpaymentaddrencoding]: https://zips.z.cash/protocol/protocol.pdf#orchardpaymentaddrencoding
    pub fn from_raw_address_bytes(bytes: &[u8; 43]) -> CtOption<Self> {
        DiversifiedTransmissionKey::from_bytes(bytes[11..].try_into().unwrap()).map(|pk_d| {
            let d = Diversifier::from_bytes(bytes[..11].try_into().unwrap());
            Self::from_parts(d, pk_d)
        })
    }

    /// Encodes this address as Bech32m
    pub fn to_bech32m(&self) -> String
    {
        bech32::encode("za", self.to_raw_address_bytes().to_base32(), Variant::Bech32m).unwrap()
    }

    /// Parse a Bech32m encoded address
    pub fn from_bech32m(str: &String) -> CtOption<Self>
    {
        let (hrp, data, variant) = bech32::decode(&str).unwrap();
        let bytes: [u8; 43] = Vec::<u8>::from_base32(&data).unwrap()[0..43].try_into().expect("from_bech32m: incorrect length");
        assert_eq!(hrp, "za");
        assert_eq!(variant, Variant::Bech32m);
        Address::from_raw_address_bytes(&bytes)
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use proptest::prelude::*;

    use crate::keys::{
        testing::{arb_diversifier_index, arb_spending_key},
        FullViewingKey, Scope,
    };

    use super::Address;

    prop_compose! {
        /// Generates an arbitrary payment address.
        pub(crate) fn arb_address()(sk in arb_spending_key(), j in arb_diversifier_index()) -> Address {
            let fvk = FullViewingKey::from(&sk);
            fvk.address_at(j, Scope::External)
        }
    }

    use rand::rngs::OsRng;

    #[test]
    fn test_bech32m_encode_decode()
    {
        let mut rng = OsRng.clone();
        let a = Address::dummy(&mut rng);
        let encoded = a.to_bech32m();
        println!("{}", encoded);
        let decoded = Address::from_bech32m(&encoded).unwrap();
        assert_eq!(a.to_raw_address_bytes(), decoded.to_raw_address_bytes());
    }
}
