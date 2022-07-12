# Notes (UTXOs)

ZEOS notes are private representations of unspent transaction outputs (UTXOs). The ZEOS protocol supports three different kinds of notes:

- Fungible Token (FT)
- Non-Fungible Token (NFT)
- Authentication Token (AT)

All three note types share the same data structure:

- header [64 bit]
- address [???]
- d1 [64 bit]
- d2 [64 bit]
- sc [64 bit]
- nft [1 bit]
- rho [32 byte]
- rseed [???]
- memo [512 byte]