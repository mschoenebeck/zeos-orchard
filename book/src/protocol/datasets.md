# Global Data Sets
The ZEOS smart contract maintains the following global data sets which define the state of the UTXO transaction model.

## Commitment Tree
The UTXO Commitment Tree is a merkle tree managed by the ZEOS smart contract. It is instantiated with the [Sinsemilla](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) hash function, which can be efficiently implemented in Halo 2 arithmetic circuits. The leaves of the tree contain all existing and thus valid UTXO commitments. Thus, each UTXO that is newly created either by minting or transfer has an associated commitment leaf in the merkle tree. The number of leaves in the tree defines the size of the so-called *anonymity set*, which is directly determined by the depth of the tree.

<img align="center" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/protocol/tree.png?raw=true">

The tree is always filled from left to right. Depending on the depth of the tree, there is room for a certain amount of leaves per tree. When a tree is full, it 'overflows' to the right into a new, empty tree. Each time one or more leaves are added, the root of the tree is updated.

### Commitment Tree Root Set
In addition to the tree itself, the ZEOS smart contract maintains a set of merkle nodes in which all valid tree roots that have ever been created are being recorded. This way, the anonymity set of the protocol can be easily configured via the merkle tree depth, without having to worry about how many trees exist at a given time or to which tree a certain leaf (aka UTXO commitment) belongs.

For a UTXO to be valid, the following must be true: There exists (or existed) a merkle path that leads (or led) to a valid merkle root. This must be proven via zero knowledge proof for a UTXO in order to be spent.

The Commitment Tree itself as well as the Commitment Tree Roots set are both part of the global UTXO transaction state.

## Nullifier Set
The UTXO Nullifier Set maintains a list of all nullifiers that have been publicly exposed. Each nullifier invalidates a particular UTXO which prevents double spending. The validity of nullifiers must be proven via zero knowledge proof in order to be able to spend UTXOs.

The Nullifier Set is part of the global UTXO transaction state.

## Transmitted UTXO Ciphertext List
This global list of transmitted UTXO ciphertexts is used for the [In-band secret distribution of UTXOs](in-band.md). It is not part of the global UTXO transaction state and its usage is optional.
