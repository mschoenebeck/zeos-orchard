# ZEOS Actions
Analogous to EOSIO/Antelope actions ZEOS privacy actions (aka zactions) change the global state of the UTXO transaction model, specifically of the [Commitment Tree](datasets.md#commitment-tree), [Commitment Tree Root Set](datasets.md#commitment-tree-root-set) and the [Nullifier Set](datasets.md#nullifier-set). The only exceptions are [MINTAT](zactions/mintat.md) and [BURNAT](zactions/burnat.md) which are used to mint and burn authenticator tokens and only change the state of the third party smart contract they are associated with (see [Private Deposits & Withdrawals](private-deposits-withdrawals.md) for more details).

Based on the constraint system of the ZEOS Orchard action circuit $C_{zeos}$ the following privacy actions (i.e. zactions) are defined:

- [MINTFT](zactions/mintft.md)
- [MINTNFT](zactions/mintnft.md)
- [MINTAT](zactions/mintat.md)
- [TRANSFERFT](zactions/transferft.md)
- [TRANSFERNFT](zactions/transfernft.md)
- [BURNFT](zactions/burnft.md)
- [BURNFT2](zactions/burnft2.md)
- [BURNNFT](zactions/burnnft.md)
- [BURNAT](zactions/burnat.md)
