# UTXOs (aka Notes)
While there is only one fungible token in Zcash - the native cryptocurrency ZEC - there is significantly more token diversity in EOSIO/Antelope ecosystems. The ZEOS Orchard Shielded Protocol defines three different UTXO types:

- **Fungible Token (FT)**
  Fungible tokens on EOSIO/Antelope blockchains are defined by the *amount*, *symbol*, and *code* of the smart contract (aka EOS account name) that issues them.

- **Non-Fungible Token (NFT)**
  NFTs on EOSIO/Antelope Blockchains mainly follow the [AtomicAssets](https://github.com/pinknetworkx/atomicassets-contract) token standard and are thus defined by a unique *identifier* and *code* of the smart contract (aka EOS account name) that issues them. The latter is the atomicassets smart contract in most cases, but can also be a custom NFT contract that follows the same standard.

- **Authenticator Token (AT)**
  These tokens represent a *permission* to access specific assets in custody of a specific smart contract. They are characterized by the *code* of the respective smart contract and their $\NoteCommit$ value.

## Tuple
To cover all three ZEOS token types, the Zcash Orchard Note Tuple (as defined in Section 3.2 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf) is extended to the following structure:

- $\mathsf{d}$ is the diversifier of the recipient’s shielded payment address
- $\DiversifiedTransmitPublic$ is the diversified transmission key of the recipient’s shielded payment address
- $\mathsf{d1}$ is an integer representing either the *amount* of the UTXO (fungible token) or the lower 64 bits of an *identifier* (non-fungible token)
- $\mathsf{d2}$ is an integer representing either the *symbol* of the UTXO (fungible token) or the upper 64 bits of an *identifier* (non-fungible token)
- $\mathsf{sc}$ is an integer representing the *code* of the issuing smart contract
- $\mathsf{nft}$ is a boolean determining if this UTXO is a fungible token (FT) or non-fungible token (NFT or AT)
- $\rho$ is used to derive the nullifier of the UTXO
- $\psi$ is additional randomness used in deriving the nullifier
- $\mathsf{rcm}$ is a random commitment trapdoor as defined in section 4.1.8 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)

Note: In addition to the above listed attributes each UTXO struct contains a header field (64 bit) and a memo field (512 bytes).

## Commitment
When UTXOs are created (through minting or transfer), only a cryptographic *commitment* called $\NoteCommit$ to the tupel attributes listed above is publicly disclosed and added to a global data structure called *Commitment Tree*. This allows the sensitive information such as amount, symbol, and recipient of the UTXO to be kept secret, while the commitment is used by the zk-SNARK proof to verify that the UTXO's secret information is valid.

Since the UTXO tuple has been extended for the ZEOS Orchard Shielded Protocol, the definition of the UTXO Commitment must also be modified. The original $\NoteCommit$ is defined in section 5.4.8.4 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf). It is changed to:

$$\NoteCommit_{\mathsf{rcm}}^{\mathsf{Orchard}}(\DiversifiedTransmitBaseRepr, \DiversifiedTransmitPublicRepr, \mathsf{d1}, \rho, \psi, \mathsf{d2}, \mathsf{sc}, \mathsf{nft}) := \SinsemillaCommit_{\mathsf{rcm}}(\textsf{"z.cash:Orchard-NoteCommit"}, 
  \DiversifiedTransmitBaseRepr \bconcat
  \DiversifiedTransmitPublicRepr \bconcat
  \ItoLEBSP{64}(\mathsf{d1}) \bconcat
  \ItoLEBSP{\BaseLength{Orchard}}(\rho) \bconcat
  \ItoLEBSP{\BaseLength{Orchard}}(\psi) \bconcat
  \ItoLEBSP{64}(\mathsf{d2}) \bconcat
  \ItoLEBSP{1}(\mathsf{nft}) \bconcat
  \ItoLEBSP{64}(\mathsf{sc}))$$

## Nullifier
Each UTXO has a unique *nullifier* which is deterministically derived from the UTXO tuple. Spending a UTXO invalidates it by publicly revealing the associated nullifier and adding it to the global set of all nullifiers called *Nullifier Set*. Analogous to the UTXO commitment, this way the sensitive information (amount, symbol, receiver) of the UTXO can be kept secret, while the nullifier is used by the zk-SNARK proof to check whether the nullifier is valid. That is, whether it actually nullifies an existing valid UTXO. Furthermore, the smart contract must check if the nullifier does not yet exist in the global set of all nullifiers to avoid double spends.

The exact function for deriving the nullifier is defined in section 4.16 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf).

$$\mathsf{DeriveNullifier_{nk}}(\rho, \psi, \mathsf{cm}) = \mathsf{Extract_{\mathbb{P}}}\big([(\mathsf{PRF_{nk}^{nfOrchard}}(\rho) + \psi) \bmod q_{\mathbb{P}}] \mathcal{K}^\mathsf{Orchard} + \mathsf{cm}\big)$$

No modification is required, since it depends only on the UTXO randomness as well as its commitment. The latter has already been adapted to the new UTXO tuple structure (see previous section).
