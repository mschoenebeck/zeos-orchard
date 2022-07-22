# ZEOS Action Circuit

As in Zcash Orchard there is only one circuit which is used to generate proofs for all private actions. The ZEOS Orchard circuit is very similar to the Zcash circuit. It can be divided in three parts: A, B and C. Each circuit part represents a note and the action circuit describes their relationship to each other. There are two main configurations for this circuit.

It is either:

1. $$A = B + C$$
$$TRANSFERFT, TRANSFERNFT, BURNFT, BURNFT2, BURNNFT$$

or 

2. $$A = C = 0$$
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

Note A (transaction input):

1. $\mathsf{path}$
2. $\mathsf{pos}$
3. $\DiversifiedTransmitBase_a$
4. $\DiversifiedTransmitPublic_a$
5. $\mathsf{d1}_a$
6. $\mathsf{d2}_a$
7. $\rho_a$
8. $\psi_a$
9. $\mathsf{rcm}_a$
10. $\mathsf{cm}_a$
11. $\alpha$
12. $\mathsf{ak}$
13. $\mathsf{nk}$
14. $\mathsf{rivk}$

Note B (transaction output):

15. $\DiversifiedTransmitBase_b$
16. $\DiversifiedTransmitPublic_b$
17. $\mathsf{d1}_b$
18. $\mathsf{d2}_b$
19. $\mathsf{sc}_b$
20. $\rho_b$
21. $\psi_b$
22. $\mathsf{rcm}_b$

Note C (transaction output):

23. $\DiversifiedTransmitBase_c$
24. $\DiversifiedTransmitPublic_c$
25. $\mathsf{d1}_c$
26. $\psi_c$
27. $\mathsf{rcm}_c$

## Public Inputs
The following list contains all public inputs to the top level ZEOS action circuit.

TODO: DEFINE SYMBOLS

1. $\mathsf{ANCHOR}$
2. $\mathsf{NF}$
3. $\mathsf{RK}$
4. $\mathsf{NFT}$
5. $\mathsf{B}_{d1}$
6. $\mathsf{B}_{d2}$
7. $\mathsf{B}_{sc}$
8. $\mathsf{C}_{d1}$
9. $\mathsf{CM}_B$
10. $\mathsf{CM}_C$

## Internal Helper Signals
The following signals are circuit-internal only. They are defined as helpers for the following equations and expressions in this document.

TODO: DEFINE SYMBOLS

1. $\mathsf{root}$
2. $\mathsf{cm}_a'$
3. $\DiversifiedTransmitPublic_a'$
4. $\mathsf{rk}$
5. $\mathsf{nf}_a$
6. $\mathsf{cm}_b$
7. $\mathsf{cm}_c$

## Constraints
The following statements for private and public inputs of the ZEOS action circuit must hold. All statements have to be expressed in form of an equation evaluating to zero.

For the global statement 'either $A = B + C$ or $A = C = 0$' the following constraint for the note values ($\mathsf{d1}_a, \mathsf{d1}_b, \mathsf{d1}_c$) must hold:

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

For circuit part C the following constraints must hold:

13. $$\mathsf{NFT} \cdot \mathsf{d1}_{c} = 0$$
14. $$\mathsf{C}_{d1} \cdot (\mathsf{C}_{d1} - \mathsf{d1}_c) = 0$$
15. $$\mathsf{CM}_C \cdot (\mathsf{CM}_C - \mathsf{cm}_c) = 0$$

## Configurations
The variables $val$, $sym$ and $sc$ represent non-zero input values.

$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|c|c|}
\hline
Private Action & \mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C & Private Inputs \\\hline
MINTFT/BURNAUTH & 0 & 0 & 0 & 0 & 0 & val & sym & sc & 0 & \mathsf{cm}_b & 0 & \mathsf{d1}_a = 0, \mathsf{d1}_c = 0, \mathsf{d1}_b = val, \mathsf{d2}_b = sym, \mathsf{sc}_b = sc \\\hline
MINTNFT & 0 & 0 & 0 & 0 & 1 & val & sym & sc & 0 & \mathsf{cm}_b & 0 & \mathsf{d1}_a = 0, \mathsf{d1}_c = 0, \mathsf{d1}_b = val, \mathsf{d2}_b = sym, \mathsf{sc}_b = sc \\\hline
TRANSFERFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & \mathsf{cm}_c & \mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c \\\hline
TRANSFERNFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 1 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & 0 & \mathsf{d1}_a = \mathsf{d1}_b,  \mathsf{d1}_c = 0 \\\hline
BURNFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & val & sym & sc & 0 & 0 & \mathsf{cm}_c & \mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c, \mathsf{d1}_b = val, \mathsf{d2}_b = sym, \mathsf{sc}_b = sc \\\hline
BURNFT2 & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & val_b & sym & sc & val_c & 0 & 0 & \mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c, \mathsf{d1}_b = val_b, \mathsf{d2}_b = sym, \mathsf{sc}_b = sc, \mathsf{d1}_c = val_c \\\hline
BURNNFT & \mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 1 & val & sym & sc & 0 & 0 & 0 & \mathsf{d1}_a = \mathsf{d1}_b, \mathsf{d1}_c = 0, \mathsf{d1}_b = val, \mathsf{d2}_b = sym, \mathsf{sc}_b = sc \\\hline
\end{array}
$

### MINTFT/BURNAUTH
<img align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/B.png?raw=true">
Note: The private actions 'MINTFT' and 'BURNAUTH' share the exact same circuit configuration.

$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C \\\hline
0 & 0 & 0 & 0 & 0 & val & sym & sc & 0 & \mathsf{cm}_b & 0 \\\hline
\end{array}
$

Given:\
$\mathsf{ANCHOR} = \mathsf{NF} = \mathsf{RK_x} = \mathsf{RK_y} = 0 $\
$ \Rightarrow $\
$\mathsf{d1}_a = 0$\
because of constraints (2), (5), (6) and (8).

Given:\
$\mathsf{B}_{d1} = val ≠ 0 $\
$ \Rightarrow $\
$\mathsf{d1}_b = \mathsf{B}_{d1} = val$\
because of constraint (9).

Given:\
$\mathsf{B}_{d2} = sym ≠ 0 $\
$ \Rightarrow $\
$\mathsf{d2}_b = \mathsf{B}_{d2} = sym$\
because of constraint (10).

Given:\
$\mathsf{B}_{sc} = sc ≠ 0 $\
$ \Rightarrow $\
$\mathsf{sc}_b = \mathsf{B}_{sc} = sc$\
because of constraint (11).

Given:\
$\mathsf{d1}_a = 0, \mathsf{d1}_b ≠ 0 $\
$ \Rightarrow $\
$\mathsf{d1}_c = 0$\
because of constraint (1).

Given:\
$\mathsf{CM}_B ≠ 0 $\
$ \Rightarrow $\
$\mathsf{CM}_B = \mathsf{cm}_b$\
because of constraint (12).

### MINTNFT
<img align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/B.png?raw=true">

$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C \\\hline
0 & 0 & 0 & 0 & 1 & val & sym & sc & 0 & \mathsf{cm}_b & 0 & \\\hline
\end{array}
$

Given:\
$\mathsf{ANCHOR} = \mathsf{NF} = \mathsf{RK_x} = \mathsf{RK_y} = 0 $\
$ \Rightarrow $\
$\mathsf{d1}_a = 0$\
because of constraints (2), (5), (6) and (8).

Given:\
$\mathsf{B}_{d1} = val ≠ 0 $\
$ \Rightarrow $\
$\mathsf{d1}_b = \mathsf{B}_{d1} = val$\
because of constraint (9).

Given:\
$\mathsf{B}_{d2} = sym ≠ 0 $\
$ \Rightarrow $\
$\mathsf{d2}_b = \mathsf{B}_{d2} = sym$\
because of constraint (10).

Given:\
$\mathsf{B}_{sc} = sc ≠ 0 $\
$ \Rightarrow $\
$\mathsf{sc}_b = \mathsf{B}_{sc} = sc$\
because of constraint (11).

Given:\
$\mathsf{NFT} = \mathsf{d1}_a = 0, \mathsf{d1}_b ≠ 0 $\
$ \Rightarrow $\
$\mathsf{d1}_c = 0$\
because of constraints (1) and (13).

Given:\
$\mathsf{CM}_B ≠ 0 $\
$ \Rightarrow $\
$\mathsf{CM}_B = \mathsf{cm}_b$\
because of constraint (12).

### TRANSFERFT
<img align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">

$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C \\\hline
\mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & \mathsf{cm}_c \\\hline
\end{array}
$

Given:\
$\mathsf{ANCHOR} ≠ 0, \mathsf{NF} ≠ 0, \mathsf{RK}_x ≠ 0, \mathsf{RK}_y ≠ 0$\
$ \Rightarrow $\
$\mathsf{d1}_a ≠ 0$\
because of constraints (2), (4), (5) and (8).

Given:\
$\mathsf{d1}_a ≠ 0$\
$ \Rightarrow $\
$\mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c$\
$\mathsf{d2}_a = \mathsf{d2}_b$\
because of constraints (1) and (7).

Given:\
$\mathsf{d1}_a ≠ 0$\
$ \Rightarrow $\
$\mathsf{ANCHOR} = root $\
because of constraint (2).

Given:\
$\mathsf{d1}_a ≠ 0$\
$ \Rightarrow $\
$\mathsf{NF} = \mathsf{nf}_a = \rho_b$\
because of constraint (8).

Given:\
$\mathsf{d1}_a ≠ 0$\
$ \Rightarrow $\
$\mathsf{RK}_{x/y} = \mathsf{rk}_{x/y}$\
because of constraints (4) and (5).

Given:\
$\mathsf{CM}_B ≠ 0 $\
$ \Rightarrow $\
$\mathsf{CM}_B = \mathsf{cm}_b$\
because of constraint (12).

Given:\
$\mathsf{CM}_C ≠ 0 $\
$ \Rightarrow $\
$\mathsf{CM}_C = \mathsf{cm}_c$\
because of constraint (15).

### TRANSFERNFT
<img align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/AB.png?raw=true">

$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C \\\hline
\mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 1 & 0 & 0 & 0 & 0 & \mathsf{cm}_b & 0 \\\hline
\end{array}
$

Given:\
$\mathsf{ANCHOR} ≠ 0, \mathsf{NF} ≠ 0, \mathsf{RK}_x ≠ 0, \mathsf{RK}_y ≠ 0$\
$ \Rightarrow $\
$\mathsf{d1}_a ≠ 0$\
because of constraints (2), (4), (5) and (8).

Given:\
$\mathsf{NFT} = 1 $\
$ \Rightarrow $\
$\mathsf{d1}_c = 0$\
because of constraint (13).

Given:\
$\mathsf{d1}_a ≠ 0, \mathsf{d1}_c = 0$\
$ \Rightarrow $\
$\mathsf{d1}_a = \mathsf{d1}_b$\
$\mathsf{d2}_a = \mathsf{d2}_b$\
because of constraints (1) and (7).

Given:\
$\mathsf{d1}_a ≠ 0$\
$ \Rightarrow $\
$\mathsf{ANCHOR} = root $\
because of constraint (2).

Given:\
$\mathsf{d1}_a ≠ 0$\
$ \Rightarrow $\
$\mathsf{NF} = \mathsf{nf}_a = \rho_b$\
because of constraint (8).

Given:\
$\mathsf{d1}_a ≠ 0$\
$ \Rightarrow $\
$\mathsf{RK}_{x/y} = \mathsf{rk}_{x/y}$\
because of constraints (4) and (5).

Given:\
$\mathsf{CM}_B ≠ 0 $\
$ \Rightarrow $\
$\mathsf{CM}_B = \mathsf{cm}_b$\
because of constraint (12).

### BURNFT
<img align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">

$
\begin{array}{|c|c|c|c|c|c|c|c|c|c|c|}
\hline
\mathsf{ANCHOR} & \mathsf{NF} & \mathsf{RK_x} & \mathsf{RK_y} & \mathsf{NFT} & \mathsf{B}_{d1} & \mathsf{B}_{d2} & \mathsf{B}_{sc} & \mathsf{C}_{d1} & \mathsf{CM_B} & \mathsf{CM}_C \\\hline
\mathsf{root} & \mathsf{nf}_a & \mathsf{rk}_x & \mathsf{rk}_y & 0 & val & sym & sc & 0 & 0 & \mathsf{cm}_c \\\hline
\end{array}
$

\mathsf{d1}_a = \mathsf{d1}_b + \mathsf{d1}_c, \mathsf{d1}_b = val, \mathsf{d2}_b = sym, \mathsf{sc}_b = sc

### BURNFT2
<img align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">
TODO

### BURNNFT
<img align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/AB.png?raw=true">
TODO
