# Modifications
However, for the ZEOS Orchard Shielded Protocol, the Zcash Orchard action circuit needs to be slightly adapted. Firstly, because there are no transparent transactions and therefore no transparent value pool in ZEOS. Furthermore, there is not only one, but countless different value pools in ZEOS because of the token diversity in EOSIO/Antelope ecosystems. Finally, there are numerous different fungible tokens as well as NFTs, which have no value pool at all because of their uniqueness.

Therefore the following approach is chosen:

1. $\mathsf{note_{old}}$ is renamed to $\mathsf{note_{a}}$
2. $\mathsf{note_{new}}$ is renamed to $\mathsf{note_{b}}$
3. the difference value $\mathsf{v_{net}}$ is converted into a third UTXO $\mathsf{note_{c}}$
3. the $\mathsf{ValueCommit}$ of $\mathsf{v_{net}}$ then becomes a $\NoteCommit$ of $\mathsf{note_{c}}$

The previous equation of the Zcash Orchard action circuit thus becomes:

$$\mathsf{note_a.d1 = note_b.d1 + note_c.d1}$$

where $\mathsf{.d1}$ refers to the value (FT) or ID (NFT) of a UTXO as defined in [UTXO Tuple](../notes.md#tuple).

Interestingly, the above equation works for both fungible and non-fungible tokens: In case of fungible token transfers, $\mathsf{note_c.d1}$ represents the change that (normally) goes back into the sender's wallet. In case of NFT transfers, however, $\mathsf{note_c.d1}$ is necessarily zero, since an NFT cannot be split. But if $\mathsf{note_c.d1}$ equals zero, it follows from the above equation that $\mathsf{note_a.d1 = note_b.d1}$ which corresponds exactly to the desired NFT transfer from $\mathsf{note_a}$ to $\mathsf{note_b}$.

In addition to above equations, the following conditions must hold:

$$\mathsf{note_a.d2 = note_b.d2 = note_c.d2}$$
$$\mathsf{note_a.sc = note_b.sc = note_c.sc}$$

The equality of all $\mathsf{.d2}$ values enforces that either all UTXOs in the circuit must have the same token symbol (FT) or that the upper 64 bits of an ID must match (in case of NFT, but then $\mathsf{note_c.d2}$ must equal zero). The equality of all $\mathsf{.sc}$ values enforces that all UTXOs in the circuit represent tokens issued by the same smart contract.

However, introducing $\mathsf{note_c}$ and replacing the $\mathsf{ValueCommit}$ with a $\NoteCommit$ comes with the trade-off that value surpluses of multiple actions of the same transaction can no longer be balanced with each other, since a $\NoteCommit$ is no Pedersen commitment and therefore does not have homomorphic properties. This means that in the ZEOS Orchard Shielded Protocol every single action must be balanced out to zero surplus using $\mathsf{note_c}$, i.e. the inputs of an action ($\mathsf{note_a}$) must equal the outputs of the same action ($\mathsf{note_b + note_c}$).