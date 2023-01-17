# Preliminaries

This work is based on a thorough study of the following protocols and applications:
- [Nightfall](https://github.com/EYBlockchain/nightfall): A protocol for private peer-to-peer transactions on Ethereum based on zk-SNARK.
- Zcash Sprout
- Zcash Sapling
- Zcash Orchard

Related Research: 
- [Monero](https://www.getmonero.org/get-started/what-is-monero/): A protocol for private peer-to-peer transactions based on Stealth Addresses, Ring Signatures, and RingCT.
- [Dero](https://dero.io/): A protocol for private peer-to-peer transactions based on Homomorphic Encryption.

## Zcash Protocol Specification
Since the protocol presented here is a fork of the Zcash Orchard Shielded Protocol the same specification applies almost everywhere. Only differences/extensions of the original protocol are specified here. Thus there are a lot of references to the original [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf).

## ZEOS Whitepaper
The [ZEOS whitepaper](https://github.com/mschoenebeck/zeos-docs/releases/download/v1.0.0/zeos_whitepaper_v1.0.0.pdf) contains a first version of the concepts specified here. However, it was written in regards to the [Groth16](https://www.zeroknowledgeblog.com/index.php/groth16) proving system while the protocol presented here is based on the [Halo2](https://halo2.dev/) proving system. The concepts described there are still mostly valid though.

## Terminology
The terminology used is based on that of Nightfall and Zcash. The abbreviation 'UTXO' means 'Unspent Transaction Output' and is used interchangably with the term 'note'. The terms 'mint' and 'burn' refer to the creation (mint) or nullification (burn) of UTXOs (aka notes). The term 'ZEOS smart contract' refers to an EOSIO/Antelope smart contract that implements the ZEOS Orchard Shielded Protocol as specified here.

## Introduction Video
The following video gives a short introduction to the features of the Zcash Orchard Shielded Protocol by the leading developers Sean Bowe and Daira Hopwood.

[![IMAGE ALT TEXT HERE](https://img.youtube.com/vi/acl_RjBUoRE/0.jpg)](https://www.youtube.com/watch?v=acl_RjBUoRE)