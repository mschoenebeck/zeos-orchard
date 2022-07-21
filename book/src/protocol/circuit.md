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

## Public Inputs

## Constraints
- A
- B
- C

## Configurations
- Table

### MINTFT/BURNAUTH
### MINTNFT
### TRANSFERFT
### TRANSFERNFT
### BURNFT
### BURNFT2
### BURNNFT