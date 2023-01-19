# Private Deposits and Withdrawals
Based on the [ZActions](zactions.md) defined in the previous section it is now possible to specify a concept for private interactions with EOSIO/Antelope smart contracts using the ZEOS wallet instead of EOSIO/Antelope accounts. The following sections specify the processes of private token deposits and withdrawals involving a third party smart contract. Check out pages 22 to 26 of the [ZEOS Whitepaper](https://github.com/mschoenebeck/zeos-docs/releases/download/v1.0.0/zeos_whitepaper_v1.0.0.pdf) for a first introduction of the concept.

## Private Deposit of fungible tokens
The way token deposits work on EOSIO/Antelope blockchains is by using notification handler. This is a callback function which is executed by the receiving smart contract on incoming token transfers. Within the handler function, it can then be determined from which EOS account the transaction was sent and thus the token amount can be assigned to the corresponding user.

In order to do such a deposit privately using a ZEOS wallet the [BURNFT](zactions/burnft.md) (or [BURNNFT](zactions/burnnft.md) for NFTs) action is being utilized. Executing this action effectively moves an asset from a private ZEOS wallet (i.e. from custody of the ZEOS smart contract) into an EOSIO/Antelope account specified by the user. If the receiving account has a smart contract with a notification handler deployed it will be able to handle the deposit in almost the same way traditional deposits work.

### Flow
The following steps describe the flow of actions for a private deposit into a third party smart contract on EOSIO/Antelope.

#### Step 0
The (non-)fungible asset to be deposited to the third party smart contract is being held in a private ZEOS wallet.

#### Step 1
The [MINTAT](zactions/mintat.md) action is executed to create a new authenticator token. The UTXO created by this action, $\mathsf{note_{auth}}$, has the $\NoteCommit$ value $\mathsf{cm_{auth}}$. As specified in MINTAT the attribute $\mathsf{note_{auth}.\mathsf{sc}}$ is set to the EOSIO/Antelope account (code) of the third party smart contract.

#### Step 2
Encode the 32 bytes long $\NoteCommit$ value of the authenticator token $\mathsf{cm_{auth}}$ as a string. Using a memory efficient encoding like base85 this will result in an ASCII string of exactly 40 bytes.

$$\mathsf{str_{cm_{auth}}} := \mathsf{base85.encode}(\mathsf{cm_{auth}})$$

#### Step 3
The [BURNFT](zactions/burnft.md) (or [BURNNFT](zactions/burnnft.md)) action is executed to move the (non-)fungible asset to be deposited from the private ZEOS wallet into the account of the third party smart contract. The 40 bytes long string-encoded $\NoteCommit$ value of the previously minted authenticator token $\mathsf{str_{cm_{auth}}}$ is being used as a *memo* for the EOSIO/Antelope transfer action resulting from the [BURNFT](zactions/burnft.md) ([BURNNFT](zactions/burnnft.md)) action.

#### Step 4
The notification handler of the receiving third party smart contract now needs to distinguish between incoming transfers from either the ZEOS smart contract or other EOSIO/Antelope accounts:

If the sending EOSIO/Antelope account is the ZEOS smart contract:
- Read the memo field and decode the first 40 bytes which contains $\mathsf{str_{cm_{auth}}}$ to retrieve the $\NoteCommit$ value of the user's authenticator token:
$$\mathsf{cm_{auth}} := \mathsf{base85.decode}(\mathsf{str_{cm_{auth}}})$$
- Use the lower 64 bits of $\mathsf{cm_{auth}}$ as primary key to book the (non-)fungible asset to be deposited in a seperate multi-index table for private deposits (i.e. not the same table used for traditional token deposits via EOSIO/Antelope accounts)

Else:
- Book the deposited (non-)fungible asset using the EOSIO/Antelope account name in the conventional manner (traditional EOSIO/Antelope token deposit)

Note that for security reasons it is important to use two different multi-index tables: One for traditional token deposits (i.e. transparent EOSIO/Antelope deposits) and one for private token deposits. That is to prevent attackers from creating an EOSIO/Antelope account with the exact lower 64 Bits of an authenticator token's $\NoteCommit$ value and thus being able to illegitimately withdraw privately deposited assets.

## Private Withdrawal of fungible tokens
In order to privately withdraw assets from a third party smart contract back into a private ZEOS wallet a private withdrawal action needs to be implemented by the third party smart contract. This action takes the following parameters:
- The public inputs $x_\mathsf{auth}$ of a [BURNAT](zactions/burnat.md) action (containing the $\NoteCommit$ value of the corresponding authenticator token $\mathsf{cm_{auth}}$)
- A zero knowledge proof $\pi_{C_{zeos}, \omega_\mathsf{auth}, x_\mathsf{auth}}$ of satisfying arguments to execute [BURNAT](zactions/burnat.md)
- The public inputs $x_\mathsf{mint}$ of a [MINTFT](zactions/mintft.md) or [MINTNFT](zactions/mintnft.md) action (containing the necessary information (amount/id, symbol, code) of the corresponding asset to be withdrawn)
- A zero knowledge proof $\pi_{C_{zeos}, \omega_\mathsf{mint}, x_\mathsf{mint}}$ of satisfying arguments to execute [MINTFT](zactions/mintft.md) or [MINTNFT](zactions/mintnft.md) action

### Flow
The following steps describe the flow of actions for a private withdrawal from a third party smart contract on EOSIO/Antelope.

#### Step 0
The (non-)fungible asset to be withdrawn from the third party smart contract is being held in a multi-index table booked under the lower 64 bits of a $\NoteCommit$ value $\mathsf{cm_{auth}}$ as primary key.

#### Step 1
The private withdrawal action of the third party smart contract is executed with the above listed parameters generated by the user.

#### Step 2
Within the private withdrawal action the third party contract performs the following steps:
- Check if $x_\mathsf{auth}\mathsf{.B}_{sc}$ matches the EOSIO/Antelope account name of the third party smart contract (i.e. was the authenticator token created for this contract?)
- Extract the lower 64 bits of the $\NoteCommit$ value of the authenticator token $\mathsf{cm_{auth}}$ from the public inputs $x_\mathsf{auth}.\mathsf{CM}_B$ and use it as primary key to find the corresponding asset inside the multi-index table of privately deposited assets
- If key doesn't exist, cancel execution
- If key does exist, check if:
  - $x_\mathsf{mint}\mathsf{.B}_{d1}$ matches the deposited asset's amount (or id in case of NFT)
  - $x_\mathsf{mint}\mathsf{.B}_{d2}$ matches the deposited asset's symbol (or 0 in case of NFT)
  - $x_\mathsf{mint}\mathsf{.B}_{sc}$ matches the deposited asset's smart contract code
- Execute the [BURNAT](zactions/burnat.md) action of the ZEOS smart contract as an inline action with $(\pi_{C_{zeos}, \omega_\mathsf{auth}, x_\mathsf{auth}}, x_\mathsf{auth})$ to authorize the withdrawal (if it fails, execution cancels)
- Transfer the (non-)fungible asset to be withdrawn to the ZEOS smart contract (where it is stored in the asset buffer)
- Execute the [MINTFT](zactions/mintft.md) (or [MINTNFT](zactions/mintnft.md)) action of the ZEOS smart contract as an inline action with $(\pi_{C_{zeos}, \omega_\mathsf{mint}, x_\mathsf{mint}}, x_\mathsf{mint})$ to create the corresponding UTXO in the private ZEOS wallet of the user
