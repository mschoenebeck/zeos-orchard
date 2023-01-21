# MINTNFT
<img height="256" align="right" src="https://github.com/mschoenebeck/zeos-docs/blob/main/book/circuit/B.png?raw=true">

Mints a new non-fungible UTXO. This effectively moves a non-fungible asset from a transparent EOSIO/Antelope account into a private ZEOS wallet. For this operation only part B of the ZEOS Action Circuit $C_{zeos}$ is used since only one new UTXO is created by this action.

Since EOSIO/Antelope does not specify a standard for non-fungible tokens the token standard of [AtomicAssets](https://atomicassets.io/) is applied.

## Privacy Implications
This action provides only limited privacy protection:

- sender: **traceable** - the sender's EOSIO/Antelope account
- asset: **traceable** - asset id of the asset's smart contract's transfer¹ action
- memo: **untraceable** - hidden in UTXO ciphertext
- receiver: **untraceable** - hidden in UTXO ciphertext

¹ refers to the [transfer](https://github.com/pinknetworkx/atomicassets-contract/wiki/Actions#transfer) action of the AtomicAssets smart contract

## Flow
The following steps specify the flow of MINTNFT.

### Step 0
The non-fungible EOSIO/Antelope asset $\mathsf{b}$ to be transferred is defined by the tuple:

- $\mathsf{id}$: The id of asset $\mathsf{b}$ as specified in the [AtomicAssets developer documentation](https://github.com/pinknetworkx/atomicassets-contract/wiki/Tables#assets)
- $\mathsf{code}$: The EOSIO/Antelope account of the AtomicAssets smart contract that issues asset $\mathsf{b}$

### Step 1
Create a new UTXO tuple $\mathsf{note_b}$ representing the non-fungible asset $\mathsf{b}$:

- $\mathsf{d} =$ Diversifier index of the receiving ZEOS wallet address
- $\DiversifiedTransmitPublic =$ Public key of the receiving ZEOS wallet address
- $\mathsf{d1} = \mathsf{b.id}$
- $\mathsf{d2} = 0$
- $\mathsf{sc} = \mathsf{b.code}$
- $\mathsf{nft} = 1$
- $\rho =$ Choose a random value
- $\psi =$ Choose a random value
- $\mathsf{rcm} =$ Choose a random value

### Step 2
Calculate the diversified transmission key of the receiving ZEOS wallet address (see section 5.4.1.6 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)):

- $\DiversifiedTransmitBase_{\mathsf{note_b}} = \mathsf{DiversifyHash^{Orchard}}(\mathsf{note_b.d})$

### Step 3
Calculate the $\NoteCommit$ of $\mathsf{note_b}$ (see [UTXO Commitment](../notes.md#commitment)):

- $\mathsf{cm_{\mathsf{note_b}}} = \NoteCommit_{\mathsf{rcm}}^{\mathsf{Orchard}}(\DiversifiedTransmitBase_{\mathsf{note_b}}, \mathsf{note_b.\DiversifiedTransmitPublic}, \mathsf{note_b.d1}, \mathsf{note_b.\rho}, \mathsf{note_b.\psi, \mathsf{note_b.d2}, \mathsf{note_b.sc}, \mathsf{note_b.nft}})$

### Step 4
Set the private inputs $\omega$ of the arithmetic circuit:

- $\mathsf{path} = 0$
- $\mathsf{pos} = 0$
- $\DiversifiedTransmitBase_a = 0$
- $\DiversifiedTransmitPublic_a = 0$
- $\mathsf{d1}_a = 0$
- $\mathsf{d2}_a = 0$
- $\rho_a = 0$
- $\psi_a = 0$
- $\mathsf{rcm}_a = 0$
- $\mathsf{cm}_a = 0$
- $\alpha = 0$
- $\mathsf{ak} = 0$
- $\mathsf{nk} = 0$
- $\mathsf{rivk} = 0$
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

### Step 5
Set the public inputs $x$ of the arithmetic circuit:

- $\mathsf{ANCHOR} = 0$
- $\mathsf{NF} = 0$
- $\mathsf{RK}_X = 0$
- $\mathsf{RK}_Y = 0$
- $\mathsf{NFT} = 1$
- $\mathsf{B}_{d1} = \mathsf{note_b.d1}$
- $\mathsf{B}_{d2} = \mathsf{note_b.d2}$
- $\mathsf{B}_{sc} = \mathsf{note_b.sc}$
- $\mathsf{C}_{d1} = 0$
- $\mathsf{CM}_B = \mathsf{cm_{\mathsf{note_b}}}$
- $\mathsf{CM}_C = 0$
- $\mathsf{ACC}_B = 0$
- $\mathsf{ACC}_C = 0$

### Step 6
Generate $\pi_{C_{zeos}, \omega, x}$ a proof of knowledge of satisfying arguments $(\omega, x)$ so that $C_{zeos}(\omega, x) = 1$

The pair $(\pi_{C_{zeos}, \omega, x}, x)$ is the zk-SNARK which attests to knowledge of private inputs $\omega$ without revealing them.

### Step 7
Generate UTXO ciphertext $\mathsf{note_b}_\mathsf{enc}$ of $\mathsf{note_b}$ for the receiver of the UTXO (see section 4.19.1 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf))

### Step 8
Transfer asset $\mathsf{b}$ to the ZEOS smart contract. On reception, the ZEOS smart contract stores it in an asset buffer until MINTNFT is executed.

### Step 9
Execute the MINTNFT action of the ZEOS smart contract. This action takes the following arguments:

- $\pi_{C_{zeos}, \omega, x}$: The zero knowledge proof of satisfying arguments $(\omega, x)$
- $x$: The public inputs of the zero knowledge proof $\pi_{C_{zeos}, \omega, x}$
- $\mathsf{note_b}_\mathsf{enc}$: The UTXO ciphertext which is transmitted to the receiver of $\mathsf{note_b}$

The ZEOS smart contract then performs the following checks:

- Is the zero knowledge proof $\pi_{C_{zeos}, \omega, x}$ valid?
- Does the UTXO $\mathsf{note_b}$ represent the correct asset $\mathsf{b}$ which is held in the asset buffer? I.e. are the following statements true:
  - $\mathsf{b.id} = x.\mathsf{B}_{d1}$
  - $0 = x.\mathsf{B}_{d2}$¹
  - $\mathsf{b.code} = x.\mathsf{B}_{sc}$
- Is the NFT flag set ($x.\mathsf{NFT} = 1$)?

¹NFTs of smart contracts following the AtomicAssets standard have a 64 Bit unique identifier only, thus no upper 64 Bits.

### Step 10
If $\mathsf{true}$, the ZEOS smart contract performs the following operations:

- Add $x.\mathsf{CM_B}$, the note commitment of $\mathsf{note_b}$, to the next free leaf of the [Commitment Tree](../datasets.md#commitment-tree)
- Add the new root of the Commitment Tree to the [Commitment Tree Root Set](../datasets.md#commitment-tree-root-set)
- Add ciphertext $\mathsf{note_b}_\mathsf{enc}$ to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list)

If $\mathsf{false}$, cancel execution.