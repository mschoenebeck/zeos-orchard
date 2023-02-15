# Arithmetic Circuit
Every privacy protocol based on zk-SNARKs is built around a so-called *arithmetic circuit*, which defines the constraint system based on which all zero knowledge proofs are generated. It is essentially a mathematical function $C$ that takes a set of private inputs $\omega$ and a set of public inputs $x$ and either resolves to $1$ (valid inputs) or $0$ (invalid inputs). The arithmetic circuit forms the core of any privacy protocol and should be designed to be as optimal as possible. In Halo 2, the complexity (i.e. the size) of the circuit is directly correlated to the proof verification time as well as the proof size. Thus, the cost (i.e. blockchain resources) of private transactions is directly determined by the complexity of the arithmetic circuit.

The circuits of all three Zcash protocols - Sprout, Sapling and Orchard - are fundamentally different from each other. This is mainly due to the use of different zk-SNARK proving systems, in which cryptographic "gadgets" - i.e. arithmetic implementations of hash functions or elliptic curve cryptography - have different complexity (i.e. number of constraints).

The Zcash Orchard Shielded Protocol, for example, is based on the Halo 2 Proving System and a [PLONKish Arithmetization](https://zcash.github.io/halo2/concepts/arithmetization.html) in which lookup tables can be used. This allows the [Sinsemilla](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) hash function, which was developed specifically for this protocol, to be implemented highly efficiently within the circuit. This is crucial for the complexity of the circuit, since for each UTXO that is issued, the valid merkle path must be proven, resulting in a concatenation of multiple Sinsemilla gadgets within the circuit. Since the merkle path accounts for the vast majority of the complexity of the entire circuit, the choice of an efficiently implementable hash function is crucial.

For example, in the protocol evolution from Zcash Sprout to Zcash Sapling, the hash function used in the Merkle tree was changed from [Sha256 to Blake2s](https://github.com/zcash/zcash/issues/2258), since the latter leads to a significantly lower number of constraints in a Groth16 proving system (over the curves Bls12-381/Jubjub) and R1CS arithmetization. There are a number of interesting discussions on this topic by Zcash developers on [Github](https://github.com/zcash/zcash/issues/2233).

The arithmetic circuit is of critical importance to the protocol, as it defines exactly what a valid UTXO transaction is. In the following, we will first roughly explain the arithmetic circuit of the Zcash Orchard Shielded Protocol and then describe the changes that lead to the design of the ZEOS Orchard Shielded Protocol arithmetic circuit.

## Notation
The following notation is used to formally express arithmetic circuits and their context.

- $\omega$: The private inputs of an arithmetic circuit
- $x$: The public inputs of an arithmetic circuit
- $C$: An arithmetic circuit $C : (\omega, x) \to \lbrace 0, 1 \rbrace$
- $\pi_{C, \omega, x}$: A proof for a circuit $C$ created with arguments $(\omega, x)$