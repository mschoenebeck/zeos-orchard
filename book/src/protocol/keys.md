# Keys & Addresses

The entire underlying cryptography of the protocol is identical to Zcash Orchard and is precisely specified in the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf). The key and address derivation is thus also exactly identical to Zcash Orchard. The only difference is that there are no transparent UTXOs in the ZEOS Orchard Shielded Protocol and thus no unshielded transactions. The newly introduced concept of 'Unified Payment Addresses' in Zcash is therefore not adopted. In ZEOS there are only 'Shielded Payment Addresses'.

For a detailed explanation of all key components, please refer to the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf), section 3.1 and section 4.2.3 respectively. In this document only the most important parts are briefly discussed. All key components are 32 bytes long.

<img align="center" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/protocol/keys.png?raw=true">

## Spending Key
The Spending Key $\mathsf{sk}$ is a randomly chosen number from which all further key material is derived. Since the introduction of [BIP 32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) and [ZIP 32](https://zips.z.cash/zip-0032#specification-orchard-key-derivation), respectively, spending keys are usually derived deterministically from long, human readable seed phrases via pseudo-random function.

## Full Viewing Key
The Full Viewing Key $\mathsf{fvk}$ is the actual set of keys needed to generate private transactions. It is derived directly from the Spending Key and consists of three sub-keys:

- $\AuthSignPublic$ - Spend Validating Key
- $\NullifierKey$ - Nullifier Deriving Key
- $\mathsf{rivk}$ - $\CommitIvk$ Randomness

This key represents the minimum authority needed to spend UTXOs.

## Incoming Viewing Key
The Incoming Viewing Key $\InViewingKey$ can be used to detect incoming payments (i.e. received UTXOs), but not to spend them. This key can be used, for example, for payment terminals to be able to confirm successfully received payments to customers.

In addition, each UTXO has a unique Viewing Key that can be shared by the sender with others in order to prove that a transmitted UTXO has been received.

## Outgoing Viewing Key
The Outgoing Viewing Key $\mathsf{ovk}$ can be used to detect all sent payments (i.e. spent UTXOs) of a wallet.

## Shielded Payment Address
An Orchard Shielded Payment Address is 43 bytes long and consists of:

- $\mathsf{d}$ - Diversifier (10 bytes)
- $\DiversifiedTransmitPublic$ - Diversified Transmission Key (33 bytes)

Since Zcash Orchard it is possible to deterministically derive several diversified payment addresses from one and the same spending authority. A group of such addresses share the same Full Viewing Key, Incoming Viewing Key and Outgoing Viewing Key.