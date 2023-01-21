# BURNAT
Burns an authenticator token. The zero knowledge proof required for this action is identical to the one of the MINTNFT action. The only difference is that it does not create a new leaf in the merkle tree. Instead it just proves knowledge of the spend authority of the address and of the secret randomness the authenticator token was minted with. This gives access to privately withdraw assets from a third party smart contract where they have been deposited using the authenticator token's note commitment value as an identifier.

## Privacy Implications
No assets are being moved by this action.

## Flow
The following steps specify the flow of BURNAT.

### Step 0
The UTXO $\mathsf{note_b}$ is an authenticator token representing a permission.

### Step 1
Calculate the diversified transmission key of the ZEOS wallet address owning the permission (see section 5.4.1.6 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)):

- $\DiversifiedTransmitBase_{\mathsf{note_b}} = \mathsf{DiversifyHash^{Orchard}}(\mathsf{note_b.d})$

### Step 2
Calculate the $\NoteCommit$ of $\mathsf{note_b}$ (see [UTXO Commitment](../notes.md#commitment)):

- $\mathsf{cm_{\mathsf{note_b}}} = \NoteCommit_{\mathsf{rcm}}^{\mathsf{Orchard}}(\DiversifiedTransmitBase_{\mathsf{note_b}}, \mathsf{note_b.\DiversifiedTransmitPublic}, \mathsf{note_b.d1}, \mathsf{note_b.\rho}, \mathsf{note_b.\psi, \mathsf{note_b.d2}, \mathsf{note_b.sc}, \mathsf{note_b.nft}})$

### Step 3
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

### Step 4
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

### Step 5
Generate $\pi_{C_{zeos}, \omega, x}$ a proof of knowledge of satisfying arguments $(\omega, x)$ so that $C_{zeos}(\omega, x) = 1$

The pair $(\pi_{C_{zeos}, \omega, x}, x)$ is the zk-SNARK which attests to knowledge of private inputs $\omega$ without revealing them.

### Step 6
Generate UTXO ciphertext $\mathsf{note_b}^\mathsf{burn}_\mathsf{enc}$ of $\mathsf{note_b}$ which is burned and therefore has the BURN flag set. This ciphertext is created only for the sender's wallet transaction history to detect burned UTXOs when scanning the ZEOS smart contract's state (see section 4.19.1 of the [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)).

### Step 7
Execute the BURNAT action of the ZEOS smart contract. This action takes the following arguments:

- $\pi_{C_{zeos}, \omega, x}$: The zero knowledge proof of satisfying arguments $(\omega, x)$
- $x$: The public inputs of the zero knowledge proof $\pi_{C_{zeos}, \omega, x}$
- $\mathsf{note_b}^\mathsf{burn}_\mathsf{enc}$: The UTXO ciphertext which indicates the 'burned' permission $\mathsf{note_b}$

The ZEOS smart contract then performs the following checks:

- Is the zero knowledge proof $\pi_{C_{zeos}, \omega, x}$ valid?
- Is the NFT flag set ($x.\mathsf{NFT} = 1$)?

### Step 8
If $\mathsf{true}$, the ZEOS smart contract performs the following operations:

- Add ciphertext $\mathsf{note_b}^\mathsf{burn}_\mathsf{enc}$ to the [Transmitted UTXO Ciphertext List](../datasets.md#transmitted-utxo-ciphertext-list)

If $\mathsf{false}$, cancel execution.