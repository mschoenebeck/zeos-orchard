# Overview

The ZEOS Orchard Shielded Protocol specifies a non-traceable UTXO transaction model implemented in an EOSIO/Antelope smart contract. This means that private ZEOS wallets do not exist directly in blockchain RAM - like EOS accounts, for example - but only in the form of UTXOs that are assigned to specific wallet addresses. Users can therefore - as with any legacy cryptocurrency - simply create a new ZEOS wallet by picking a large random number (aka private key).

All assets held in private ZEOS wallets are actually in custody of the ZEOS smart contract. In order to transfer an asset from a transparent EOS account to a private ZEOS wallet, it is actually transferred to the ZEOS smart contract, while at the same time a (private) UTXO is minted, assigned to a private ZEOS wallet address specified by the user. UTXOs can then be transferred completely anonymously between ZEOS wallets without their ownership being traceable on chain.

The only data managed by the ZEOS smart contract are cryptographic [*commitments*](protocol/notes.md#commitment) of valid UTXOs and their [*nullifiers*](protocol/notes.md#nullifier). The validity of commitments and nullifiers is proven to the public using *zero knowledge proofs* (more precisely: [zk-SNARKs](https://z.cash/technology/zksnarks/)) without revealing any sensitive information about the private UTXOs themselves.

Analogous to minting, UTXOs can also be burned in order to transfer assets from a private ZEOS wallets back to a transparent EOS accounts. By burning the corresponding UTXO, the underlying asset is freed from custody of the ZEOS smart contract and transferred to an EOS account specified by the user. This way, assets can be freely move between EOS accounts and ZEOS wallets.

<img align="center" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/protocol/account_wallet_movement.png?raw=true">

Privacy therefore only exists for assets "inside" the ZEOS application - i.e. only for assets in custody of the ZEOS smart contract and represented by UTXOs in private ZEOS wallets. The underlying EOSIO/Antelope blockchain remains transparent, of course. However, the introduction of a so-called *authenticator token* enables private token deposits and withdrawals. This means that users are able to interact directly from their ZEOS wallets with other smart contracts of the same EOSIO/Antelope blockchain - as long as those contracts implement the necessary smart contract interface of the ZEOS Orchard Shielded Protocol.

The introduction of the authenticator token and the resulting private deposits and withdrawals make the ZEOS Orchard Shielded Protocol a truly universal privacy protocol for EOSIO/Antelope blockchains that can offer users of private ZEOS wallets almost the same blockchain experience as users of transparent EOS accounts - but with complete privacy.

See the [ZEOS Whitepaper](https://github.com/mschoenebeck/zeos-docs/releases/download/v1.0.0/zeos_whitepaper_v1.0.0.pdf) for more information about the concepts of the protocol presented here.