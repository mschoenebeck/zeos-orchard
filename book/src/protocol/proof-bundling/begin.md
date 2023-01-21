# begin
This action initiates a sequence of [step](step.md) actions.

## Parameters
- $\Pi_{C_{zeos}, \Omega, X}$: A proof bundle validating all zactions within this EOSIO/Antelope transaction
- $\mathsf{tx}$: A sequence of EOSIO/Antelope actions to be executed within this EOSIO/Antelope transaction
- $\mathsf{notes_{enc}}$: A sequence of encrypted UTXO ciphertexts to be added to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list) as part of the [In-band secret distribution of UTXOs](../in-band.md#in-band-secret-distribution-of-utxos)

## Flow
The following steps specify the flow of 'begin'.

### Step 0
The 'begin' action is called as part of an EOSIO/Antelope transaction.

### Step 1
The currently executing EOSIO/Antelope transaction is scanned by looping through all actions contained in this transaction. The following checks are performed:
- There is only one 'begin' action within the entire EOSIO/Antelope transaction
- The number of 'step' actions within the transaction equals the number of actions contained in the sequence $\mathsf{tx}$ passed as parameter
- The sequence of 'step' actions is continuous and succeed the 'begin' action

### Step 2
Loop through all actions within the sequence $\mathsf{tx}$ and perform the following steps:
- Is this action whitelisted by the ZEOS smart contract? If $\mathsf{false}$: cancel execution
- Does this action depend on a sequence of zactions? If $\mathsf{true}$:
  - Loop through all zactions of this action and collect public inputs $x$ by adding them to the set of public inputs $X$

### Step 3
Verify the Halo2 proof bundle $\Pi_{C_{zeos}, \Omega, X}$ with the set of public inputs $X$ collected in the previous step. If proof verification fails, cancel execution.

### Step 4
Check if the number of encrypted UTXO ciphertexts in $\mathsf{notes_{enc}}$ matches the number of expected UTXO ciphertexts based on the zactions in this transaction. If so, add the sequence $\mathsf{notes_{enc}}$ to the global set of [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list).

### Step 5
Add all UTXO commitments $\mathsf{CM}_B$ and $\mathsf{CM}_C$ contained in the set of public inputs $X$ which are not related to authenticator tokens to the [Commitment Tree](../datasets.md#commitment-tree). Add the new root of the tree to the [Commitment Tree Root Set](../datasets.md#commitment-tree-root-set).

### Step 6
Copy the sequence of EOSIO/Antelope actions $\mathsf{tx}$ into the ZEOS smart contract's *transaction buffer*. This is a singleton which buffers the sequence of actions, $\mathsf{tx}, for actual execution in the upcoming 'step' actions.