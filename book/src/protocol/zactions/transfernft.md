# TRANSFERNFT
<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/AB.png?raw=true">

Transfers a non-fungible UTXO. This effectively moves a non-fungible asset from one private ZEOS wallet to another. For this operation only part A and B of the ZEOS Action Circuit $C_{zeos}$ is used since one existing UTXO is being spent while one new UTXO is being created by this action.

## Privacy Implications
This action provides full privacy protection:

- sender: **untraceable** - hidden in zk-SNARK
- asset: **untraceable** - hidden in UTXO ciphertext
- memo: **untraceable** - hidden in UTXO ciphertext
- receiver: **untraceable** - hidden in UTXO ciphertext

## Flow
The following steps specify the flow of TRANSFERNFT.

### Step 0
The UTXO $\mathsf{note_a}$ represents a non-fungible EOSIO/Antelope asset to be transmitted.

### Step 1
Create a new UTXO tuple $\mathsf{note_b}$ representing the non-fungible asset $\mathsf{note_a}$ at the receiving address with new randomness:

- $\mathsf{d} =$ Diversifier index of the receiving ZEOS wallet address
- $\DiversifiedTransmitPublic =$ Public key of the receiving ZEOS wallet address
- $\mathsf{d1} = \mathsf{note_a.d1}$
- $\mathsf{d2} = \mathsf{note_a.d2}$
- $\mathsf{sc} = \mathsf{note_a.sc}$
- $\mathsf{nft} = 1$
- $\rho =$ Choose a random value
- $\psi =$ Choose a random value
- $\mathsf{rcm} =$ Choose a random value

### Step 2
Calculate the diversified transmission keys of all ZEOS wallet addresses involved (see section 5.4.1.6 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)):

- $\DiversifiedTransmitBase_{\mathsf{note_a}} = \mathsf{DiversifyHash^{Orchard}}(\mathsf{note_a.d})$
- $\DiversifiedTransmitBase_{\mathsf{note_b}} = \mathsf{DiversifyHash^{Orchard}}(\mathsf{note_b.d})$

### Step 3
Calculate the [Commitment](../notes.md#commitment) of the two UTXOs $\mathsf{note_a}$ and $\mathsf{note_b}$:

- $\mathsf{cm_{\mathsf{note_a}}} = \NoteCommit_{\mathsf{rcm}}^{\mathsf{Orchard}}(\DiversifiedTransmitBase_{\mathsf{note_a}}, \mathsf{note_a.\DiversifiedTransmitPublic}, \mathsf{note_a.d1}, \mathsf{note_a.\rho}, \mathsf{note_a.\psi, \mathsf{note_a.d2}, \mathsf{note_a.sc}, \mathsf{note_a.nft}})$
- $\mathsf{cm_{\mathsf{note_b}}} = \NoteCommit_{\mathsf{rcm}}^{\mathsf{Orchard}}(\DiversifiedTransmitBase_{\mathsf{note_b}}, \mathsf{note_b.\DiversifiedTransmitPublic}, \mathsf{note_b.d1}, \mathsf{note_b.\rho}, \mathsf{note_b.\psi, \mathsf{note_b.d2}, \mathsf{note_b.sc}, \mathsf{note_b.nft}})$

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
- $\DiversifiedTransmitBase_b = \DiversifiedTransmitBase_{\mathsf{note_b}}$
- $\DiversifiedTransmitPublic_b = \mathsf{note_b.\DiversifiedTransmitPublic}$
- $\mathsf{d1}_b = \mathsf{note_b.d1}$
- $\mathsf{d2}_b = \mathsf{note_b.d2}$
- $\mathsf{sc}_b = \mathsf{note_b.sc}$
- $\rho_b = \mathsf{note_b.\rho}$
- $\psi_b = \mathsf{note_b.\psi}$
- $\mathsf{rcm}_b = \mathsf{note_b.rcm}$
- $\mathsf{acc}_b = 0$
- $\DiversifiedTransmitBase_c = 0$
- $\DiversifiedTransmitPublic_c = 0$
- $\mathsf{d1}_c = 0$
- $\psi_c = 0$
- $\mathsf{rcm}_c = 0$
- $\mathsf{acc}_c = 0$

### Step 8
Set the public inputs $x$ of the arithmetic circuit:

- $\mathsf{ANCHOR} = \mathsf{root}$
- $\mathsf{NF} = \mathsf{nf_a}$
- $\mathsf{RK}_X = \mathsf{rk'}_x$
- $\mathsf{RK}_Y = \mathsf{rk'}_y$
- $\mathsf{NFT} = 1$
- $\mathsf{B}_{d1} = 0$
- $\mathsf{B}_{d2} = 0$
- $\mathsf{B}_{sc} = 0$
- $\mathsf{C}_{d1} = 0$
- $\mathsf{CM}_B = \mathsf{cm_{\mathsf{note_b}}}$
- $\mathsf{CM}_C = 0$
- $\mathsf{ACC}_B = 0$
- $\mathsf{ACC}_C = 0$

### Step 9
Generate $\pi_{C_{zeos}, \omega, x}$ a proof of knowledge of satisfying arguments $(\omega, x)$ so that $C_{zeos}(\omega, x) = 1$

The pair $(\pi_{C_{zeos}, \omega, x}, x)$ is the zk-SNARK which attests to knowledge of private inputs $\omega$ without revealing them.

### Step 10
Generate UTXO ciphertext $\mathsf{note_b}_\mathsf{enc}$ of $\mathsf{note_b}$ for the receiver of the UTXOs (see section 4.19.1 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf))

### Step 11
Execute the TRANSFERNFT action of the ZEOS smart contract. This action takes the following arguments:

- $\pi_{C_{zeos}, \omega, x}$: The zero knowledge proof of satisfying arguments $(\omega, x)$
- $x$: The public inputs of the zero knowledge proof $\pi_{C_{zeos}, \omega, x}$
- $\mathsf{note_b}_\mathsf{enc}$: The UTXO ciphertext which is transmitted to the receiver of $\mathsf{note_b}$

The ZEOS smart contract then performs the following checks:

- Is the zero knowledge proof $\pi_{C_{zeos}, \omega, x}$ valid?

### Step 12
If $\mathsf{true}$, the ZEOS smart contract performs the following operations:

- Add $x.\mathsf{NF}$, the nullifier of $\mathsf{note_a}$ to the [Nullifier Set](../datasets.md#nullifier-set)
- Add $x.\mathsf{CM_B}$, the note commitment of $\mathsf{note_b}$, to the next free leaf of the [Commitment Tree](../datasets.md#commitment-tree)
- Add the new root of the Commitment Tree to the [Commitment Tree Root Set](../datasets.md#commitment-tree-root-set)
- Add ciphertext $\mathsf{note_b}_\mathsf{enc}$ to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list)

If $\mathsf{false}$, cancel execution.