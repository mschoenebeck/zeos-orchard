# BURNFT
<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/ABC.png?raw=true">

Burns (part of) a fungible UTXO. This effectively moves a certain balance of a fungible asset from a private ZEOS wallet to a transparent EOSIO/Antelope account. For this operation the entire ZEOS Action Circuit $C_{zeos}$ is used since one existing UTXO is being spent, part of its balance is publicly revealed and one new UTXO is being created by this action.

## Privacy Implications
This action provides only limited privacy protection:

- sender: **untraceable** - hidden in zk-SNARK
- asset: **traceable** - quantity of the asset's smart contract's transfer action
- memo: **traceable** - memo of the asset's smart contract's transfer action
- receiver: **traceable** - the receiver's EOSIO/Antelope account

## Flow
The following steps specify the flow of BURNFT.

### Step 0
The UTXO $\mathsf{note_a}$ represents an amount of a fungible EOSIO/Antelope asset from which a certain (partial) $\mathsf{amount_b}$ is to be transmitted to the EOSIO/Antelope account $\mathsf{account_b}$. Therefore the following must apply: $\mathsf{note_a.d1} >= \mathsf{amount_b}$.

### Step 1
Create a new UTXO tuple $\mathsf{note_c}$ representing the 'change' to $\mathsf{amount_b}$ of $\mathsf{note_a}$:

- $\mathsf{d} =$ Diversifier index of the sending¹ ZEOS wallet address
- $\DiversifiedTransmitPublic =$ Public key of the sending¹ ZEOS wallet address
- $\mathsf{d1} = \mathsf{note_a.d1} - \mathsf{amount_b}$
- $\mathsf{d2} = \mathsf{note_a.d2}$
- $\mathsf{sc} = \mathsf{note_a.sc}$
- $\mathsf{nft} = 0$
- $\rho =$ Choose a random value
- $\psi =$ Choose a random value
- $\mathsf{rcm} =$ Choose a random value

¹ Note that it is also possible to choose a third wallet address for $\mathsf{note_c}$ instead of the sender's address.

### Step 2
Calculate the diversified transmission keys of all ZEOS wallet addresses involved (see section 5.4.1.6 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)):

- $\DiversifiedTransmitBase_{\mathsf{note_a}} = \mathsf{DiversifyHash^{Orchard}}(\mathsf{note_a.d})$
- $\DiversifiedTransmitBase_{\mathsf{note_c}} = \mathsf{DiversifyHash^{Orchard}}(\mathsf{note_c.d})$

### Step 3
Calculate the [Commitment](../notes.md#commitment) of the two UTXOs $\mathsf{note_a}$ and $\mathsf{note_c}$:

- $\mathsf{cm_{\mathsf{note_a}}} = \NoteCommit_{\mathsf{rcm}}^{\mathsf{Orchard}}(\DiversifiedTransmitBase_{\mathsf{note_a}}, \mathsf{note_a.\DiversifiedTransmitPublic}, \mathsf{note_a.d1}, \mathsf{note_a.\rho}, \mathsf{note_a.\psi, \mathsf{note_a.d2}, \mathsf{note_a.sc}, \mathsf{note_a.nft}})$
- $\mathsf{cm_{\mathsf{note_c}}} = \NoteCommit_{\mathsf{rcm}}^{\mathsf{Orchard}}(\DiversifiedTransmitBase_{\mathsf{note_c}}, \mathsf{note_c.\DiversifiedTransmitPublic}, \mathsf{note_c.d1}, \mathsf{note_c.\rho}, \mathsf{note_c.\psi, \mathsf{note_c.d2}, \mathsf{note_c.sc}, \mathsf{note_c.nft}})$

### Step 4
Calculate the $\mathsf{root}$ of the [Commitment Tree](../datasets.md#commitment-tree) based on the sister path of $\mathsf{cm_{\mathsf{note_a}}}$.

### Step 5
Calculate the [Nullifier](../notes.md#nullifier) $\mathsf{nf_a}$ of $\mathsf{note_a}$:

- $\mathsf{nf_a} = \mathsf{DeriveNullifier_{nk}}(\mathsf{note_a.\rho}, \mathsf{note_a.\psi}, \mathsf{cm_{\mathsf{note_a}}})$

### Step 6
Choose a random value $\alpha'$ and calculate the Spend Authority $\mathsf{rk'}$ for this action (see section 4.17.4 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)):

- $\alpha' = $ Choose a random value
- $\mathsf{rk'} = \mathsf{SpendAuthSig^{Orchard}.RandomizePublic}(\alpha', \mathsf{ak}^{\mathbb{P}})$

where $\mathsf{ak}$ is the Spend Validating Key of $\mathsf{note_a}$ which is part of the [Full Viewing Key](../keys.md#full-viewing-key).

### Step 7
Set the private inputs $\omega$ of the arithmetic circuit:

- $\mathsf{path} = $ sister path of $\mathsf{cm_{\mathsf{note_a}}}$ in the [Commitment Tree](../datasets.md#commitment-tree)
- $\mathsf{pos} = $ leaf index of $\mathsf{cm_{\mathsf{note_a}}}$ in the [Commitment Tree](../datasets.md#commitment-tree)
- $\DiversifiedTransmitBase_a = \DiversifiedTransmitBase_{\mathsf{note_a}}$
- $\DiversifiedTransmitPublic_a = \mathsf{note_a.\DiversifiedTransmitPublic}$
- $\mathsf{d1}_a = \mathsf{note_a.d1}$
- $\mathsf{d2}_a = \mathsf{note_a.d2}$
- $\rho_a = \mathsf{note_a.\rho}$
- $\psi_a = \mathsf{note_a.\psi}$
- $\mathsf{rcm}_a = \mathsf{note_a.rcm}$
- $\mathsf{cm}_a = \mathsf{cm_{\mathsf{note_a}}}$
- $\alpha = \alpha'$
- $\mathsf{ak} = $ Spend Validating Key of $\mathsf{note_a}$ which is part of the [Full Viewing Key](../keys.md#full-viewing-key)
- $\mathsf{nk} = $ Nullifier Deriving Key of $\mathsf{note_a}$ which is part of the [Full Viewing Key](../keys.md#full-viewing-key)
- $\mathsf{rivk} = $ $\CommitIvk$ Randomness of $\mathsf{note_a}$ which is part of the [Full Viewing Key](../keys.md#full-viewing-key)
- $\DiversifiedTransmitBase_b = 0$
- $\DiversifiedTransmitPublic_b = 0$
- $\mathsf{d1}_b = \mathsf{amount_b}$
- $\mathsf{d2}_b = \mathsf{note_a.d2}$
- $\mathsf{sc}_b = \mathsf{note_a.sc}$
- $\rho_b = \mathsf{nf_a}$
- $\psi_b = 0$
- $\mathsf{rcm}_b = 0$
- $\mathsf{acc}_b = \mathsf{account_b}$
- $\DiversifiedTransmitBase_c = \DiversifiedTransmitBase_{\mathsf{note_c}}$
- $\DiversifiedTransmitPublic_c = \mathsf{note_c.\DiversifiedTransmitPublic}$
- $\mathsf{d1}_c = \mathsf{note_c.d1}$
- $\psi_c = \mathsf{note_c.\psi}$
- $\mathsf{rcm}_c = \mathsf{note_c.rcm}$
- $\mathsf{acc}_c = 0$

### Step 8
Set the public inputs $x$ of the arithmetic circuit:

- $\mathsf{ANCHOR} = \mathsf{root}$
- $\mathsf{NF} = \mathsf{nf_a}$
- $\mathsf{RK}_X = \mathsf{rk'}_x$
- $\mathsf{RK}_Y = \mathsf{rk'}_y$
- $\mathsf{NFT} = 0$
- $\mathsf{B}_{d1} = \mathsf{amount_b}$
- $\mathsf{B}_{d2} = \mathsf{note_a.d2}$
- $\mathsf{B}_{sc} = \mathsf{note_a.sc}$
- $\mathsf{C}_{d1} = 0$
- $\mathsf{CM}_B = 0$
- $\mathsf{CM}_C = \mathsf{cm_{\mathsf{note_c}}}$
- $\mathsf{ACC}_B = \mathsf{account_b}$
- $\mathsf{ACC}_C = 0$

### Step 9
Generate $\pi_{C_{zeos}, \omega, x}$ a proof of knowledge of satisfying arguments $(\omega, x)$ so that $C_{zeos}(\omega, x) = 1$

The pair $(\pi_{C_{zeos}, \omega, x}, x)$ is the zk-SNARK which attests to knowledge of private inputs $\omega$ without revealing them.

### Step 10
Generate UTXO ciphertext $\mathsf{note_c}_\mathsf{enc}$ of $\mathsf{note_c}$ for the receiver of the UTXO (see section 4.19.1 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf))

Generate an additional UTXO ciphertext $\mathsf{note_b}^\mathsf{burn}_\mathsf{enc}$ of $\mathsf{amount_b}$ which is burned and therefore has the BURN flag set. This ciphertext is created only for the sender's wallet transaction history to detect burned UTXOs when scanning the ZEOS smart contract's state.

### Step 11
Execute the BURNFT action of the ZEOS smart contract. This action takes the following arguments:

- $\pi_{C_{zeos}, \omega, x}$: The zero knowledge proof of satisfying arguments $(\omega, x)$
- $x$: The public inputs of the zero knowledge proof $\pi_{C_{zeos}, \omega, x}$
- $\mathsf{note_c}_\mathsf{enc}$: The UTXO ciphertext which is transmitted to the receiver of $\mathsf{note_c}$
- $\mathsf{note_b}^\mathsf{burn}_\mathsf{enc}$: The UTXO ciphertext which indicates the 'burned' $\mathsf{amount_b}$

The ZEOS smart contract then performs the following checks:

- Is the zero knowledge proof $\pi_{C_{zeos}, \omega, x}$ valid?

### Step 12
If $\mathsf{true}$, the ZEOS smart contract performs the following operations:

- Add $x.\mathsf{NF}$, the nullifier of $\mathsf{note_a}$ to the [Nullifier Set](../datasets.md#nullifier-set)
- Add $x.\mathsf{CM_C}$, the note commitment of $\mathsf{note_c}$, to the next free leaf of the [Commitment Tree](../datasets.md#commitment-tree)
- Add the new root of the Commitment Tree to the [Commitment Tree Root Set](../datasets.md#commitment-tree-root-set)
- Add ciphertext $\mathsf{note_c}_\mathsf{enc}$ to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list)
- Add ciphertext $\mathsf{note_b}^\mathsf{burn}_\mathsf{enc}$ to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list)
- Transfer $\mathsf{amount_b}$ of asset represented by $\mathsf{note_a}$ into the EOSIO/Antelope account $\mathsf{account_b}$.

If $\mathsf{false}$, cancel execution.