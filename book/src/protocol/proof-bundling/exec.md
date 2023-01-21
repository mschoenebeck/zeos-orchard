# exec
This action executes a sequence of [ZActions](../zactions.md).

## Parameters
- $\mathsf{ztx}$: A sequence of [zaction tuples](../proof-bundling.md#zactions-as-eosioantelope-action-parameters)

## Flow
The following steps specify the flow of 'exec'.

### Step 0
The 'exec' action is always executed as an inline action of a [step](step.md) action which always follows the execution of a [begin](begin.md), where the proof bundle is verified which validates all subsequent zactions.

### Step 1
Let $\mathsf{za}$ loop through the sequence of zactions $\mathsf{ztx}$ and check $\mathsf{za.type}$:

- if [MINTFT](../zactions/mintft.md) execute [step 9](../zactions/mintft.md#step-9) and [step 10](../zactions/mintft.md#step-10) of the corresponding action flow:
  - *The zero knowledge proof is already verified by the [begin](begin.md) action*
  - Do the public inputs of this zaction represent the correct asset $\mathsf{b}$ which is held in the asset buffer? I.e. are the following statements true:
    - $\mathsf{b.amount} = \mathsf{za}.x.\mathsf{B}_{d1}$
    - $\mathsf{b.symbol} = \mathsf{za}.x.\mathsf{B}_{d2}$
    - $\mathsf{b.code} = \mathsf{za}.x.\mathsf{B}_{sc}$
  - Check if the NFT flag is unset ($x.\mathsf{NFT} = 0$)?
  - *The UTXO commitment is already added to the [Commitment Tree](../datasets.md#commitment-tree) by the [begin](begin.md) action*
  - *The new root of the Commitment Tree is already added to the [Commitment Tree Root Set](../datasets.md#commitment-tree-root-set) by the [begin](begin.md) action*
  - *The UTXO ciphertext is already added to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list) by the [begin](begin.md) action*

- if [MINTNFT](../zactions/mintnft.md) execute [step 9](../zactions/mintnft.md#step-9) and [step 10](../zactions/mintnft.md#step-10) of the corresponding action flow:
  - *The zero knowledge proof is already verified by the [begin](begin.md) action*
  - Do the public inputs of this zaction represent the correct asset $\mathsf{b}$ which is held in the asset buffer? I.e. are the following statements true:
    - $\mathsf{b.id} = \mathsf{za}.x.\mathsf{B}_{d1}$
    - $0 = \mathsf{za}.x.\mathsf{B}_{d2}$
    - $\mathsf{b.code} = \mathsf{za}.x.\mathsf{B}_{sc}$
  - Check if the NFT flag is set ($x.\mathsf{NFT} = 1$)?
  - *The UTXO commitment is already added to the [Commitment Tree](../datasets.md#commitment-tree) by the [begin](begin.md) action*
  - *The new root of the Commitment Tree is already added to the [Commitment Tree Root Set](../datasets.md#commitment-tree-root-set) by the [begin](begin.md) action*
  - *The UTXO ciphertext is already added to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list) by the [begin](begin.md) action*

- if [MINTAT](../zactions/mintat.md) execute [step 4](../zactions/mintat.md#step-4) of the corresponding action flow:
  - *The UTXO ciphertext is already added to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list) by the [begin](begin.md) action*

- if [TRANSFERFT](../zactions/transferft.md) execute [step 11](../zactions/transferft.md#step-11) and [step 12](../zactions/transferft.md#step-12) of the corresponding action flow:
  - *The zero knowledge proof is already verified by the [begin](begin.md) action*
  - Check if the NFT flag is unset ($x.\mathsf{NFT} = 0$)?
  - Add $\mathsf{za}.x.\mathsf{NF}$ to the [Nullifier Set](../datasets.md#nullifier-set)
  - *The UTXO commitments are already added to the [Commitment Tree](../datasets.md#commitment-tree) by the [begin](begin.md) action*
  - *The new root of the Commitment Tree is already added to the [Commitment Tree Root Set](../datasets.md#commitment-tree-root-set) by the [begin](begin.md) action*
  - *The UTXO ciphertexts are already added to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list) by the [begin](begin.md) action*

- if [TRANSFERNFT](../zactions/transfernft.md) execute [step 11](../zactions/transfernft.md#step-11) and [step 12](../zactions/transfernft.md#step-12) of the corresponding action flow:
  - *The zero knowledge proof is already verified by the [begin](begin.md) action*
  - Check if the NFT flag is set ($x.\mathsf{NFT} = 1$)?
  - Add $\mathsf{za}.x.\mathsf{NF}$ to the [Nullifier Set](../datasets.md#nullifier-set)
  - *The UTXO commitment is already added to the [Commitment Tree](../datasets.md#commitment-tree) by the [begin](begin.md) action*
  - *The new root of the Commitment Tree is already added to the [Commitment Tree Root Set](../datasets.md#commitment-tree-root-set) by the [begin](begin.md) action*
  - *The UTXO ciphertext is already added to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list) by the [begin](begin.md) action*

- if [BURNFT](../zactions/burnft.md) execute [step 11](../zactions/burnft.md#step-11) and [step 12](../zactions/burnft.md#step-12) of the corresponding action flow:
  - *The zero knowledge proof is already verified by the [begin](begin.md) action*
  - Check if the NFT flag is unset ($x.\mathsf{NFT} = 0$)?
  - Add $\mathsf{za}.x.\mathsf{NF}$ to the [Nullifier Set](../datasets.md#nullifier-set)
  - *The UTXO commitment is already added to the [Commitment Tree](../datasets.md#commitment-tree) by the [begin](begin.md) action*
  - *The new root of the Commitment Tree is already added to the [Commitment Tree Root Set](../datasets.md#commitment-tree-root-set) by the [begin](begin.md) action*
  - *The UTXO ciphertext is already added to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list) by the [begin](begin.md) action*
  - Continue to loop $\mathsf{ztx}$ with $\mathsf{za}$ and perform above steps to sum up all the amounts $\mathsf{za}.x.\mathsf{B}_{d1}$ while:
    - $\mathsf{za.type}$ equals BURNFT
    - $\mathsf{za}.x.\mathsf{B}_{d2}$ remains the same (i.e. same token symbol)
    - $\mathsf{za}.x.\mathsf{B}_{sc}$ remains the same (i.e. same token contract)
  - Transfer the sum of all amounts as one EOSIO/Antelope 'transfer' action of smart contract $\mathsf{za}.x.\mathsf{B}_{sc}$ using $\mathsf{za}.\mathsf{memo}$ as the memo into the EOSIO/Antelope account $\mathsf{za}.x.\mathsf{ACC}_{B}$. The loop ensures that burning multiple UTXOs of the same currency is combined into one EOSIO/Antelope transfer action.

- if [BURNFT2](../zactions/burnft2.md) execute [step 10](../zactions/burnft2.md#step-10) and [step 11](../zactions/burnft2.md#step-11) of the corresponding action flow:
  - TODO: Not yet finalized.

- if [BURNNFT](../zactions/burnnft.md) execute [step 10](../zactions/burnnft.md#step-10) and [step 11](../zactions/burnnft.md#step-11) of the corresponding action flow:
  - *The zero knowledge proof is already verified by the [begin](begin.md) action*
  - Check if the NFT flag is set ($x.\mathsf{NFT} = 1$)?
  - Add $\mathsf{za}.x.\mathsf{NF}$ to the [Nullifier Set](../datasets.md#nullifier-set)
  - *The UTXO ciphertext is already added to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list) by the [begin](begin.md) action*
  - Transfer the non-fungible asset ($\mathsf{za}.x.\mathsf{B}_{d1}$, $\mathsf{za}.x.\mathsf{B}_{d2}$, $\mathsf{za}.x.\mathsf{B}_{sc}$) into the EOSIO/Antelope account $\mathsf{za}.x.\mathsf{ACC}_{B}$.

- if [BURNAT](../zactions/burnat.md) execute [step 7](../zactions/burnat.md#step-7) and [step 8](../zactions/burnat.md#step-8) of the corresponding action flow:
  - *The zero knowledge proof is already verified by the [begin](begin.md) action*
  - Check if the NFT flag is set ($x.\mathsf{NFT} = 1$)?
  - *The UTXO ciphertext is already added to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list) by the [begin](begin.md) action*