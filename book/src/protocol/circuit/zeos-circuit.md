# ZEOS Action Circuit
<img align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">

As in Zcash Orchard there is only one circuit which is used to generate proofs for all privacy actions. The ZEOS Orchard circuit, $C_{zeos}$, is very similar to the Zcash Orchard circuit. It can be divided into three parts: A, B and C. Each circuit part represents a UTXO and the action circuit describes their relationship to each other. There are two main configurations for this circuit.

It is either:

1. $$A = B + C$$

or 

2. $$A = C = 0$$

The first configuration is used for all transfer and burn actions. UTXO $A$ represents the note which is being spent by the action. UTXO $B$ represents the receiving part of the action whereas UTXO $C$ represents the 'change' which goes usually back into the wallet of the sender (spender of UTXO $A$). Hence the relation $A = B + C$ between the UTXOs.

In case of NFT transfers (or burns) UTXO $C$ is always zero which enforces $A = B$. This statement must be true for NFT transfers since NFTs are not divisable.

The second configuration is used for minting notes only. The configuration $A = B = 0$ effectively disables the circuit parts $A$ and $C$ leaving only part $B$ enabled.

See also:
- the [schematic](https://github.com/mschoenebeck/zeos-docs/blob/main/action_circuit/action_circuit_schematic.pdf) of the action circuit (TODO: Legend)
- the [layout](https://github.com/mschoenebeck/zeos-docs/blob/main/action_circuit/action-circuit-layout.png) of the action circuit (column types explained [here](https://halo2.dev/))

## Private Inputs ($\omega$)
The following list contains all private inputs to the top level ZEOS action circuit.

Note $A$ (action input):

1. $\mathsf{path}$                  : Authentication path of note commitment A. The sister path of note commitment A which is required in order to calculate it's merkle plath.
2. $\mathsf{pos}$                   : Position of note commitment A inside the merkle tree. Specifically this is the leaf index of note commitment A.
3. $\DiversifiedTransmitBase_a$     : Address diversify hash of note A. Deterministically derived from a diversifier index (see p. 37 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).
4. $\DiversifiedTransmitPublic_a$   : Address public key of note A. Derived from Incoming Viewing Key and diversify hash (see p. 14 and 37 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).
5. $\mathsf{d1}_a$                  : Either the value of note A (fungible token) or the (lower 64 bits of the) unique identifier of note A (non-fungible token).
6. $\mathsf{d2}_a$                  : Either the symbol code of note A (fungible token) or the (upper 64 bits of the) unique identifier of note A (non-fungible token).
7. $\rho_a$                         : Randomness to derive nullifier of note A (equals nullifier of note that was spent in order to create note A) (see p. 14 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf))
8. $\psi_a$                         : Additional randomness to derive nullifier of note A (see p. 14 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf))
9. $\mathsf{rcm}_a$                 : Random commitment trapdoor of note commitment A (see p. 14 and 28 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf))
10. $\mathsf{cm}_a$                 : Note commit of note A (see p. 28 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).
11. $\alpha$                        : Randomness to derive a spend authorization signature for note A (see p. 55 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).
12. $\mathsf{ak}$                   : Spend Validating Key which is part of the Full Viewing Key components (see p. 36 and 116 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).
13. $\mathsf{nk}$                   : Nullifier Deriving Key which is part of the Full Viewing Key components (see p. 36 and 116 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).
14. $\mathsf{rivk}$                 : Randomness which is part of the Full Viewing Key components to derive corresponding Incoming Viewing Key of the address diversify hash of note A (see p. 36, 37 and 116 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).

Note $B$ (action output):

15. $\DiversifiedTransmitBase_b$    : Address diversify hash of note B.
16. $\DiversifiedTransmitPublic_b$  : Address public key of note B.
17. $\mathsf{d1}_b$                 : Either the value of note B (fungible token) or the (lower 64 bits of the) unique identifier of note B (non-fungible token).
18. $\mathsf{d2}_b$                 : Either the symbol code of note B (fungible token) or the (upper 64 bits of the) unique identifier of note B (non-fungible token).
19. $\mathsf{sc}_b$                 : The code of the smart contract issuing notes A, B and C.
20. $\rho_b$                        : Randomness to derive nullifier of note B (equals nullifier of note A).
21. $\psi_b$                        : Additional randomness to derive nullifier of note B.
22. $\mathsf{rcm}_b$                : Random commitment trapdoor of note commitment B.
23. $\mathsf{acc}_b$                : Code of EOS account in which this note is 'burned'.

Note $C$ (action output):

24. $\DiversifiedTransmitBase_c$    : Address diversify hash of note C.
25. $\DiversifiedTransmitPublic_c$  : Address public key of note C.
26. $\mathsf{d1}_c$                 : Value of note C (fungible token only).
27. $\psi_c$                        : Additional randomness to derive nullifier of note C.
28. $\mathsf{rcm}_c$                : Random commitment trapdoor of note commitment C.
29. $\mathsf{acc}_c$                : Code of EOS account in which this note is 'burned'.

## Public Inputs ($x$)
The following list contains all public inputs to the top level ZEOS action circuit.

1. $\mathsf{ANCHOR}$                : Merkle tree root of authentication path of note A (see p. 17 to 19 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).
2. $\mathsf{NF}$                    : Nullifier of note A (see p. 56 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).
3. $\mathsf{RK}_X$                  : Spend Authority (x component) of ($\alpha$, $\mathsf{ak}$) (see p. 61 [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).
4. $\mathsf{RK}_Y$                  : Spend Authority (y component).
5. $\mathsf{NFT}$                   : Indicates if notes in circuit represent NFTs or fungible tokens.
6. $\mathsf{B}_{d1}$                : Exposes value (fungible token) or unique id (non-fungible token) of note B in case of MINT or BURN.
7. $\mathsf{B}_{d2}$                : Exposes symbol (fungible token) or unique id (non-fungible token) of note B in case of MINT or BURN.
8. $\mathsf{B}_{sc}$                : Exposes smart contract code of note B in case of MINT or BURN.
9. $\mathsf{C}_{d1}$                : Exposes value of note C in case of BURNFT2 (fungible tokens only).
10. $\mathsf{CM}_B$                 : Note commitment of note B.
11. $\mathsf{CM}_C$                 : Note commitment of note C.
12. $\mathsf{ACC}_B$                : Code of EOS account in which note B is 'burned'.
13. $\mathsf{ACC}_C$                : Code of EOS account in which note C is 'burned'.

## Circuit Internal Signals
The following signals are circuit-internal only.

1. $\mathsf{root}$                  : Derived root of merkle path of note A.
2. $\mathsf{cm}_a'$                 : Derived commitment of note A.
3. $\DiversifiedTransmitPublic_a'$  : Derived address public key of note A.
4. $\mathsf{rk}_x$                  : Derived spend authority (x component) of note A.
5. $\mathsf{rk}_y$                  : Derived spend authority (y component) of note A.
6. $\mathsf{nf}_a$                  : Derived nullifier of note A.
7. $\mathsf{cm}_b$                  : Derived commitment of note B.
8. $\mathsf{cm}_c$                  : Derived commitment of note C.

## Constraints
Constraining an arithmetic circuit in Halo 2 is very similar to integrated circuit design in computer engineering. All Constraints are expressed as mathematical equations evaluating to zero. The logical AND becomes a multiplication and the logical OR becomes an addition.

The following statements for private and public inputs must hold.

The global statement 'either $A = B + C$ or $A = C = 0$' results in the following constraint for the note values $\mathsf{d1}_a, \mathsf{d1}_b, \mathsf{d1}_c$:

1. $$(\mathsf{d1}_a - \mathsf{d1}_b - \mathsf{d1}_c) \cdot (\mathsf{d1}_a + \mathsf{d1}_c) = 0$$

For circuit part A the following constraints must hold:

2. $$\mathsf{d1}_a \cdot (\mathsf{root} - \mathsf{ANCHOR}) = 0$$
3. $$\mathsf{d1}_a \cdot (\mathsf{cm}_a - \mathsf{cm}_a') = 0$$
4. $$\mathsf{d1}_a \cdot (\DiversifiedTransmitPublic_a - \DiversifiedTransmitPublic_a') = 0$$
5. $$\mathsf{d1}_a \cdot (\mathsf{rk}_x - \mathsf{RK}_x) = 0$$
6. $$\mathsf{d1}_a \cdot (\mathsf{rk}_y - \mathsf{RK}_y) = 0$$
7. $$\mathsf{d1}_a \cdot (\mathsf{d2}_a - \mathsf{d2}_b) = 0$$
8. $$\mathsf{d1}_a \cdot (\mathsf{nf}_a + \rho_b - 2 \cdot \mathsf{NF}) = 0$$

For circuit part B the following constraints must hold:

9. $$\mathsf{B}_{d1} \cdot (\mathsf{B}_{d1} - \mathsf{d1}_b) = 0$$
10. $$\mathsf{B}_{d1} \cdot (\mathsf{B}_{d2} - \mathsf{d2}_b) = 0$$
11. $$\mathsf{B}_{d1} \cdot (\mathsf{B}_{sc} - \mathsf{sc}_b) = 0$$
12. $$\mathsf{CM}_B \cdot (\mathsf{CM}_B - \mathsf{cm}_b) = 0$$
13. $$\mathsf{ACC}_B - \mathsf{acc}_b = 0$$

For circuit part C the following constraints must hold:

14. $$\mathsf{NFT} \cdot \mathsf{d1}_{c} = 0$$
15. $$\mathsf{C}_{d1} \cdot (\mathsf{C}_{d1} - \mathsf{d1}_c) = 0$$
16. $$\mathsf{CM}_C \cdot (\mathsf{CM}_C - \mathsf{cm}_c) = 0$$
17. $$\mathsf{ACC}_C - \mathsf{acc}_c = 0$$

## Valid Circuit Configurations
The following table lists the zactions and the corresponding configurations of the circuit's public inputs $x$.

$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ZAction} & \mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM}_B & \mathsf{CM}_C & \mathsf{ACC}_B & \mathsf{ACC}_C \\\hline
MINTFT & 0 & 0 & 0 & 0 & 0 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & 0 & \mathsf{cm}_b & 0 & 0 & 0 \\\hline
MINTNFT/BURNAT & 0 & 0 & 0 & 0 & 1 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & 0 & \mathsf{cm}_b & 0 & 0 & 0 \\\hline
TRANSFERFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & \mathsf{cm}_c & 0 & 0 \\\hline
TRANSFERNFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 1 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & 0 & 0 & 0 \\\hline
BURNFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & 0 & 0 & \mathsf{cm}_c & \mathsf{acc}_b & 0 \\\hline
BURNFT2 & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & \mathsf{d1}_c & 0 & 0 & \mathsf{acc}_b & \mathsf{acc}_c \\\hline
BURNNFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 1 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & 0 & 0 & 0 & \mathsf{acc}_b & 0 \\\hline
\end{array}
$

### MINTFT
$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C & \mathsf{ACC}_B & \mathsf{ACC}_C \\\hline
0 & 0 & 0 & 0 & 0 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & 0 & \mathsf{cm}_b & 0 & 0 & 0  \\\hline
\end{array}
$

<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/B.png?raw=true">

Given: $\mathsf{ANCHOR} = \mathsf{NF} = \mathsf{RK_x} = \mathsf{RK_y} = 0 $\
$\Rightarrow \mathsf{d1}_a = 0$ because of constraints (2), (5), (6) and (8)

Given: $\mathsf{CM}_B = \mathsf{cm}_b $\
$\Rightarrow \mathsf{d1}_b ≠ 0$ because of internal signal (7)

Given: $\mathsf{d1}_a = 0, \mathsf{d1}_b ≠ 0 $\
$\Rightarrow \mathsf{d1}_c = 0$ because of constraint (1)

Given: $\mathsf{ACC}_B = 0 $\
$\Rightarrow \mathsf{acc}_b = 0$ because of constraint (13)

Given: $\mathsf{ACC}_C = 0 $\
$\Rightarrow \mathsf{acc}_c = 0$ because of constraint (17)

### MINTNFT/BURNAT
Note: The actions MINTNFT and BURNAT share the exact same circuit configuration.
$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C & \mathsf{ACC}_B & \mathsf{ACC}_C \\\hline
0 & 0 & 0 & 0 & 1 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & 0 & \mathsf{cm}_b & 0 & 0 & 0 \\\hline
\end{array}
$

<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/B.png?raw=true">

Given: $\mathsf{ANCHOR} = \mathsf{NF} = \mathsf{RK_x} = \mathsf{RK_y} = 0 $\
$\Rightarrow \mathsf{d1}_a = 0$ because of constraints (2), (5), (6) and (8)

Given: $\mathsf{CM}_B = \mathsf{cm}_b $\
$\Rightarrow \mathsf{d1}_b ≠ 0$ because of internal signal (7)

Given: $\mathsf{NFT} = 1, \mathsf{d1}_a = 0, \mathsf{d1}_b ≠ 0 $\
$\Rightarrow \mathsf{d1}_c = 0$ because of constraints (1) and (14)

Given: $\mathsf{ACC}_B = 0 $\
$\Rightarrow \mathsf{acc}_b = 0$ because of constraint (13)

Given: $\mathsf{ACC}_C = 0 $\
$\Rightarrow \mathsf{acc}_c = 0$ because of constraint (17)

### TRANSFERFT
$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C & \mathsf{ACC}_B & \mathsf{ACC}_C \\\hline
\mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & \mathsf{cm}_c & 0 & 0 \\\hline
\end{array}
$

<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">

Given: $\mathsf{ANCHOR} = \mathsf{root}, \mathsf{NF} = \mathsf{nf}_a, \mathsf{RK}_x = \mathsf{rk}_x, \mathsf{RK}_y = \mathsf{rk}_y$\
$\Rightarrow \mathsf{d1}_a ≠ 0$ because of internal signals (1), (4), (5), (6) and constraints (2), (5), (6) and (8)

Given: $\mathsf{d1}_a ≠ 0$\
$\Rightarrow \mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c$ because of constraint (1)\
$\Rightarrow \mathsf{d2}_a = \mathsf{d2}_b$ because of constraint (7)\
$\Rightarrow \rho_b = \mathsf{nf}_a$ because of constraint (8)\

Given: $\mathsf{ACC}_B = 0 $\
$\Rightarrow \mathsf{acc}_b = 0$ because of constraint (13)

Given: $\mathsf{ACC}_C = 0 $\
$\Rightarrow \mathsf{acc}_c = 0$ because of constraint (17)

### TRANSFERNFT
$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C & \mathsf{ACC}_B & \mathsf{ACC}_C \\\hline
\mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 1 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & 0 & 0 & 0 \\\hline
\end{array}
$

<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/AB.png?raw=true">

Given: $\mathsf{ANCHOR} = \mathsf{root}, \mathsf{NF} = \mathsf{nf}_a, \mathsf{RK}_x = \mathsf{rk}_x, \mathsf{RK}_y = \mathsf{rk}_y$\
$\Rightarrow \mathsf{d1}_a ≠ 0$ because of internal signals (1), (4), (5), (6) and constraints (2), (5), (6) and (8)

Given: $\mathsf{NFT} = 1 $\
$\Rightarrow \mathsf{d1}_c = 0$ because of constraint (14)

Given: $\mathsf{d1}_a ≠ 0, \mathsf{d1}_c = 0$\
$\Rightarrow \mathsf{d1}_a = \mathsf{d1}_b$ because of constraint (1)

Given: $\mathsf{d1}_a ≠ 0$\
$\Rightarrow \mathsf{d2}_a = \mathsf{d2}_b$ because of constraint (7)\
$\Rightarrow \rho_b = \mathsf{nf}_a$ because of constraint (8)\

Given: $\mathsf{ACC}_B = 0 $\
$\Rightarrow \mathsf{acc}_b = 0$ because of constraint (13)

Given: $\mathsf{ACC}_C = 0 $\
$\Rightarrow \mathsf{acc}_c = 0$ because of constraint (17)

### BURNFT
$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C & \mathsf{ACC}_B & \mathsf{ACC}_C \\\hline
\mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & 0 & 0 & \mathsf{cm}_c & \mathsf{acc}_b & 0 \\\hline
\end{array}
$

<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">

Given: $\mathsf{ANCHOR} = \mathsf{root}, \mathsf{NF} = \mathsf{nf}_a, \mathsf{RK}_x = \mathsf{rk}_x, \mathsf{RK}_y = \mathsf{rk}_y$\
$\Rightarrow \mathsf{d1}_a ≠ 0$ because of internal signals (1), (4), (5), (6) and constraints (2), (5), (6) and (8)

Given: $\mathsf{d1}_a ≠ 0$\
$\Rightarrow \mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c$ because of constraint (1)\
$\Rightarrow \mathsf{d2}_a = \mathsf{d2}_b$ because of constraint (7)\
$\Rightarrow \rho_b = \mathsf{nf}_a$ because of constraint (8)\

Given: $\mathsf{ACC}_C = 0 $\
$\Rightarrow \mathsf{acc}_c = 0$ because of constraint (17)

### BURNFT2
$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C & \mathsf{ACC}_B & \mathsf{ACC}_C \\\hline
\mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & \mathsf{d1}_c & 0 & 0 & \mathsf{acc}_b & \mathsf{acc}_c \\\hline
\end{array}
$

<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">

Given: $\mathsf{ANCHOR} = \mathsf{root}, \mathsf{NF} = \mathsf{nf}_a, \mathsf{RK}_x = \mathsf{rk}_x, \mathsf{RK}_y = \mathsf{rk}_y$\
$\Rightarrow \mathsf{d1}_a ≠ 0$ because of internal signals (1), (4), (5), (6) and constraints (2), (5), (6) and (8)

Given: $\mathsf{d1}_a ≠ 0$\
$\Rightarrow \mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c$ because of constraint (1)\
$\Rightarrow \mathsf{d2}_a = \mathsf{d2}_b$ because of constraint (7)\
$\Rightarrow \rho_b = \mathsf{nf}_a$ because of constraint (8)\

### BURNNFT
$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C & \mathsf{ACC}_B & \mathsf{ACC}_C \\\hline
\mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 1 & \mathsf{d1}_b & \mathsf{d2}_b & \mathsf{sc}_b & 0 & 0 & 0 & \mathsf{acc}_b & 0 \\\hline
\end{array}
$

<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/AB.png?raw=true">

Given: $\mathsf{ANCHOR} = \mathsf{root}, \mathsf{NF} = \mathsf{nf}_a, \mathsf{RK}_x = \mathsf{rk}_x, \mathsf{RK}_y = \mathsf{rk}_y$\
$\Rightarrow \mathsf{d1}_a ≠ 0$ because of internal signals (1), (4), (5), (6) and constraints (2), (5), (6) and (8)

Given: $\mathsf{NFT} = 1 $\
$\Rightarrow \mathsf{d1}_c = 0$ because of constraint (14)

Given: $\mathsf{d1}_a ≠ 0, \mathsf{d1}_c = 0$\
$\Rightarrow \mathsf{d1}_a = \mathsf{d1}_b$ because of constraint (1)

Given $\mathsf{d1}_a ≠ 0$\
$\Rightarrow \mathsf{d2}_a = \mathsf{d2}_b$ because of constraint (7)\
$\Rightarrow \rho_b = \mathsf{nf}_a$ because of constraint (8)\

Given: $\mathsf{ACC}_C = 0 $\
$\Rightarrow \mathsf{acc}_c = 0$ because of constraint (17)
