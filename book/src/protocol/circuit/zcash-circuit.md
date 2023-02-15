# Zcash Action Circuit
A Zcash Orchard transaction is defined as a sequence of one or more *actions*. The Zcash Orchard circuit defines what a valid action is. It is important to note that in addition to shielded UTXOs, Zcash also has a transparent value pool (i.e. the Zcash protocol also allows for transparent, traceable transactions).

The general idea is as follows: Each action allows the spending of up to one UTXO ($\mathsf{note_{old}}$) and the creation of up to one new UTXO ($\mathsf{note_{new}}$). To balance the value difference between them there is a balancing value ($\mathsf{v_{net}}$).

The equation applies:

$$\mathsf{note_{old}.v} = \mathsf{note_{new}.v} + \mathsf{v_{net}}$$

where $\mathsf{.v}$ refers to the *value* (i.e. the amount of [ZEC](https://coinmarketcap.com/currencies/zcash/)) of a UTXO as defined in section 3.2 (p.14) of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf).

The value $\mathsf{v_{net}}$ thus expresses whether the UTXO transfer of a single Orchard action results in a positive or negative *value surplus*. Over the entire transaction - i.e. over the sum of all actions - $\mathsf{v_{net}}$ must obviously result in zero. This means that the sum of all inputs of a transaction must be equal to the sum of all outputs.

However, the actual value surplus of an action remains secret - just like the private inputs of the circuit (i.e. the sensitive UTXO data). Instead, only a so-called $\mathsf{ValueCommit}$ of $\mathsf{v_{net}}$ is published, which is specified in section 5.4.8.3 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf).

The $\mathsf{ValueCommit}$ is a [Pedersen Commitment](https://findora.org/faq/crypto/pedersen-commitment-with-elliptic-curves/) that has special cryptographic properties such as homomorphic addition. This property allows for the $\mathsf{v_{net}}$ values of all actions of the same transaction to be balanced without having to disclose the actual differences $\mathsf{v_{net}}$ of individual actions publicly: The homomorphically encrypted $\mathsf{ValueCommit}$ of all individual actions can be summed up and eventually balanced with a single *transparent* value from the transparent value pool (in case the sum is not zero). See section 4.14 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf) to learn more about the final balancing value of a shielded transaction.

A shielded transaction in Zcash Orchard therefore either *generates* a transparent *output* ($\mathsf{v_{net}} > 0$), or it *consumes* a transparent *input* ($\mathsf{v_{net}} < 0$), or $\mathsf{v_{net}}$ equals zero, in which case it is a fully shielded transaction with no inputs or outputs from the transparent value pool.

The flexibility of this circuit design therefore allows the four different types of actions:
1. transparent → shielded ($\mathsf{note_{old}} = 0$, resembles a *mint*).
2. shielded → transparent ($\mathsf{note_{new}} = 0$, resembles a *burn*)
3. shielded → shielded ($\mathsf{v_{net}} = 0$, resembles a shielded transfer)
4. mixed → mixed (either *mint* + shielded transfer or *burn* + shielded transfer)

The following schematic shows a highly simplified representation of the Zcash Orchard top level arithmetic circuit, reduced to the essential components. The black inputs are the *private inputs* of the arithmetic circuit and the blue inputs are the *public inputs*. Only if *all* equality gates (==) resolve to *true* the output of the circuit is $1$, otherwise it is $0$.

<img align="center" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/protocol/zcash_circuit_schematic.png?raw=true">

The circuit is divided into three areas, which are highlighted in different colors:

Area A contains all the logic regarding $\mathsf{note_{old}}$. This is by far the most complex part of the circuit, since the validity of $\mathsf{note_{old}}$ as well as the validity of $\textsf{spend\_auth}$ must be proven here. Specifically, it must be proven that:

1. There exists a sister path $\textsf{auth\_path}$ to the $\NoteCommit$ of $\mathsf{note_{old}}$, which leads to a valid $\textsf{ROOT}$ of the merkle tree (i.e. $\mathsf{note_{old}}$ is a valid UTXO).
2. The address of the UTXO $\mathsf{note_{old}}$ can be derived from $\textsf{spend\_auth}$ (i.e. $\textsf{spend\_auth}$ is indeed the correct private key to $\mathsf{note_{old}}$).
3. The nullifier is valid (i.e. $\textsf{NF}$ is indeed the nullifier of $\mathsf{note_{old}}$).

Area B proves that $\textsf{CM\_NEW}$ is indeed the correct $\NoteCommit$ to the newly created UTXO $\mathsf{note_{new}}$.

Area C proves that $\textsf{CV\_NET}$ actually represents the correct $\mathsf{ValueCommit}$ (aka Pedersen commitment) of the value surplus $\mathsf{v_{net}}$.
