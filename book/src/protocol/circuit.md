# ZEOS Action Circuit

As in Zcash Orchard there is only one circuit which is used to generate proofs for all private actions. The ZEOS Orchard circuit is very similar to the Zcash circuit. It can be divided in three parts: A, B and C. Each circuit part represents a note and the action circuit describes their relationship to each other. There are two main configurations for this circuit.

It is either:

$$A = B + C$$
$$TRANSFERFT, TRANSFERNFT, BURNFT, BURNFT2, BURNNFT$$

or 

$$A = C = 0$$
$$MINTFT, MINTNFT, BURNAUTH$$

The first configuration is used for all transer and burn actions. Note A represents the note which is being spent by the transaction. Note B represents the receiving part of the transaction whereas note C represents the 'change' which goes usually back into the wallet of the sender (spender of note A). Hence the relation $A = B + C$ between the notes.

In case of NFT transfers (or burns) note C is always zero which enforces A = B. This statement must be true for NFT transfers since they are not divisable.

The second configuration is used for minting notes only. The action BURNAUTH is on a circuit-level equivalent to minting new notes. Only on the transaction-level the data is interpreted differently to burn an authentication token instead of minting a new fungible or non-fungible token. The configuration $A = B = 0$ effectively disables the circuit parts A and C leaving only part B enabled.

For a more detailed description of the exact action circuit configuration for each specific private action types see below.

See also:
- the [schematic](https://github.com/mschoenebeck/zeos-docs/blob/main/action_circuit/action_circuit_schematic.pdf) of the action circuit
- the [layout](https://github.com/mschoenebeck/zeos-docs/blob/main/action_circuit/action-circuit-layout.png) of the action circuit

## Private Inputs
The following list contains all private inputs to the top level ZEOS action circuit.

TODO: DEFINE SYMBOLS

Transaction Input (note A):
- $\mathsf{path}$
- $\mathsf{pos}$
- $\DiversifiedTransmitBase_a$
- $\DiversifiedTransmitPublic_a$
- $\mathsf{d1}_a$
- $\mathsf{d2}_a$
- $\rho_a$
- $\psi_a$
- $\mathsf{rcm}_a$
- $\mathsf{cm}_a$
- $\alpha$
- $\mathsf{ak}$
- $\mathsf{nk}$
- $\mathsf{rivk}$

Transaction Output (note B):
- $\DiversifiedTransmitBase_b$
- $\DiversifiedTransmitPublic_b$
- $\mathsf{d1}_b$
- $\mathsf{d2}_b$
- $\mathsf{sc}_b$
- $\rho_b$
- $\psi_b$
- $\mathsf{rcm}_b$

Transaction Output (note C):
- $\DiversifiedTransmitBase_c$
- $\DiversifiedTransmitPublic_c$
- $\mathsf{d1}_c$
- $\psi_c$
- $\mathsf{rcm}_c$

## Public Inputs
The following list contains all public inputs to the top level ZEOS action circuit.

TODO: DEFINE SYMBOLS

- $\mathsf{ANCHOR}$
- $\mathsf{NF}$
- $\mathsf{RK}$
- $\mathsf{NFT}$
- $\mathsf{B}_{d1}$
- $\mathsf{B}_{d2}$
- $\mathsf{B}_{sc}$
- $\mathsf{C}_{d1}$
- $\mathsf{CM}_B$
- $\mathsf{CM}_C$

## Internal Helper Signals
The following signals are circuit-internal only. They are defined as helpers for the following equations and expressions in this document.

TODO: DEFINE SYMBOLS

- $\mathsf{root}$
- $\mathsf{cm}_a'$
- $\DiversifiedTransmitPublic_a'$
- $\mathsf{rk}$
- $\mathsf{nf}_a$
- $\mathsf{cm}_b$
- $\mathsf{cm}_c$

## Constraints
The following statements for private and public inputs of the ZEOS action circuit must hold. All statements have to be expressed in form of an equation evaluating to zero.

For the global statement 'either $A = B + C$ or $A = C = 0$' the following constraint for the note values ($\mathsf{d1}_a, \mathsf{d1}_b, \mathsf{d1}_c$) must hold:

- $(\mathsf{d1}_a - \mathsf{d1}_b - \mathsf{d1}_c) \cdot (\mathsf{d1}_a + \mathsf{d1}_c) = 0$

For circuit part A the following constraints must hold:

- $\mathsf{d1}_a \cdot (\mathsf{root} - \mathsf{ANCHOR}) = 0$
- $\mathsf{d1}_a \cdot (\mathsf{cm}_a - \mathsf{cm}_a') = 0$
- $\mathsf{d1}_a \cdot (\DiversifiedTransmitPublic_a - \DiversifiedTransmitPublic_a') = 0$
- $\mathsf{d1}_a \cdot (\mathsf{rk}_x - \mathsf{RK}_x) = 0$
- $\mathsf{d1}_a \cdot (\mathsf{rk}_y - \mathsf{RK}_y) = 0$
- $\mathsf{d1}_a \cdot (\mathsf{d2}_a - \mathsf{d2}_b) = 0$
- $\mathsf{d1}_a \cdot (\mathsf{nf}_a + \rho_b - 2 \cdot \mathsf{NF}) = 0$

For circuit part B the following constraints must hold:

- $\mathsf{B}_{d1} \cdot (\mathsf{B}_{d1} - \mathsf{d1}_b) = 0$
- $\mathsf{B}_{d1} \cdot (\mathsf{B}_{d2} - \mathsf{d2}_b) = 0$
- $\mathsf{B}_{d1} \cdot (\mathsf{B}_{sc} - \mathsf{sc}_b) = 0$
- $\mathsf{CM}_B \cdot (\mathsf{CM}_B - \mathsf{cm}_b) = 0$

For circuit part C the following constraints must hold:

- $\mathsf{NFT} \cdot \mathsf{d1}_{c} = 0$
- $\mathsf{C}_{d1} \cdot (\mathsf{C}_{d1} - \mathsf{d1}_c) = 0$
- $\mathsf{CM}_C \cdot (\mathsf{CM}_C - \mathsf{cm}_c) = 0$

## Configurations

$$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
Private Action & \mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & C {d1} & \mathsf{CM_B} & \mathsf{CM}_C & Private Inputs \\\hline
MINTFT/BURNAUTH & 0 & 0 & 0 & 0 & 0 & d1 & d2 & sc & 0 & \mathsf{cm}_b & 0 & \mathsf{d1}_a = 0, \mathsf{d1}_c = 0, \mathsf{d1}_b = d1, \mathsf{d2}_b = d2, \mathsf{sc}_b = sc \\\hline
MINTNFT & 0 & 0 & 0 & 0 & 1 & d1 & d2 & sc & 0 & \mathsf{cm}_b & 0 & \mathsf{d1}_a = 0, \mathsf{d1}_c = 0, \mathsf{d1}_b = d1, \mathsf{d2}_b = d2, \mathsf{sc}_b = sc \\\hline
TRANSFERFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & \mathsf{cm}_c & \mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c \\\hline
TRANSFERNFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 1 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & 0 & \mathsf{d1}_a = \mathsf{d1}_b,  \mathsf{d1}_c = 0 \\\hline
BURNFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & d1 & d2 & sc & 0 & 0 & \mathsf{cm}_c & \mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c, \mathsf{d1}_b = d1, \mathsf{d2}_b = d2, \mathsf{sc}_b = sc \\\hline
BURNFT2 & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & d1b & d2 & sc & d1c & 0 & 0 & \mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c, \mathsf{d1}_b = d1b, \mathsf{d2}_b = d2, \mathsf{sc}_b = sc, \mathsf{d1}_c = d1c \\\hline
BURNNFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 1 & d1 & d2 & sc & 0 & 0 & 0 & \mathsf{d1}_a = \mathsf{d1}_b, \mathsf{d1}_c = 0, \mathsf{d1}_b = d1, \mathsf{d2}_b = d2, \mathsf{sc}_b = sc \\\hline
\end{array}
$$

### MINTFT/BURNAUTH
<img align="right" height="100" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/B.png?raw=true">
TODO

### MINTNFT
<img align="right" height="100" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/B.png?raw=true">
TODO

### TRANSFERFT
<img align="right" height="100" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">
TODO

### TRANSFERNFT
<img align="right" height="100" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/AB.png?raw=true">
TODO

### BURNFT
<img align="right" height="100" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">
TODO

### BURNFT2
<img align="right" height="100" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">
TODO

### BURNNFT
<img align="right" height="100" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/AB.png?raw=true">
TODO
