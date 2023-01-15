# ZActions
Based on the constraint system of the ZEOS Orchard action circuit the following private actions (i.e. zactions) can be defined.

## MINTFT
Mints a new fungible note. This effectively moves a fungible asset from an EOS account into a ZEOS wallet.

## MINTNFT
Mints a new non-fungible note. This effectively moves a non-fungible asset from an EOS account into a ZEOS wallet.

## MINTAUTH
Mints a new authenticator token. This action does not require a zero knowledge proof. Authenticator tokens do not (directly) represent real assets and thus have no leaves in the merkle tree. Instead, the corresponding note commitment of an authenticator token is used as an identifier for third party smart contracts where assets are privately deposited. By proving knowledge of the secret randomness of an authenticator token, the deposited assets asociated with this token can be withdrawn from the third party contract at a later point in time.

## TRANSFERFT
Transfers a fungible note from one ZEOS wallet to another by spending (burning) a note (A) and minting two new notes (B and C).

## TRANSFERNFT
Transfers a non-fungible note from one ZEOS wallet to another by spending (burning) a note (A) and minting one new note (B).

## BURNFT
Burns a fungible note. This effectively moves a fungible asset from a ZEOS wallet into an EOS account.

## BURNFT2
Burns a fungible note. This effectively splits a fungible asset and moves it from a ZEOS wallet into two different EOS accounts.

## BURNNFT
Burns a non-fungible note. This effectively moves a non-fungible asset from a ZEOS wallet into an EOS account.

## BURNAUTH
Burns an authenticator token. The zero knowledge proof required for this action is identical to the one of the MINTNFT action. The only difference is that it does not create a new leaf in the merkle tree. Instead it just proves knowledge of the secret randomness the authenticator token was minted with. This gives access to privately withdraw assets from a third party smart contract where they have been deposited using the authenticator tokens note commitment value as an identifier.
