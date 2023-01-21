# ZEOS Actions (ZActions)
Analogous to EOSIO/Antelope actions ZEOS privacy actions (aka zactions) change the global state of the UTXO transaction model, specifically of the [Commitment Tree](datasets.md#commitment-tree), [Commitment Tree Root Set](datasets.md#commitment-tree-root-set) and [Nullifier Set](datasets.md#nullifier-set). The only exceptions are [MINTAT](zactions/mintat.md) and [BURNAT](zactions/burnat.md) which are used to mint and burn authenticator tokens and only change the state of the third party smart contract they are associated with (see [Private Deposits & Withdrawals](private-deposits-withdrawals.md) for more details).

## Tuple
Analogous to the EOSIO/Antelope action struct, a zaction struct is defined. The tuple contains the following elements:

- $\mathsf{type}$: The type of zaction to be executed (see [Types](#types) below)
- $x$: The public inputs of this zaction (see public inuts tuple under [ZEOS Action Circuit](circuit/zeos-circuit.md#public-inputs-x))
- $\mathsf{memo}$: A memo field used in BURN actions to set the memo for the resulting EOSIO/Antelope token transfer action

Based on this zaction tuple and in analogy to EOSIO/Antelope transactions, a ztransaction is defined as a sequence of zactions.

## Types
Based on the constraint system of the ZEOS Orchard action circuit $C_{zeos}$ the following privacy actions (i.e. zactions) are defined:

- [MINTFT](zactions/mintft.md): Mints a new UTXO representing a fungible asset.
- [MINTNFT](zactions/mintnft.md): Mints a new UTXO representing a non-fungible asset.
- [MINTAT](zactions/mintat.md): Mints a new UTXO representing a permission.
- [TRANSFERFT](zactions/transferft.md): Transfers a UTXO representing a fungible asset.
- [TRANSFERNFT](zactions/transfernft.md): Transfers a UTXO representing a non-fungible asset.
- [BURNFT](zactions/burnft.md): Burns a UTXO representing a fungible asset.
- [BURNFT2](zactions/burnft2.md): Burns a UTXO representing a fungible asset with two different receivers.
- [BURNNFT](zactions/burnnft.md): Burns a UTXO representing a non-fungible asset.
- [BURNAT](zactions/burnat.md): Burns a UTXO representing a permission.

