# ZEOS smart contract

## Tables

* * *
### vk
Maps LiquidStorage IPFS URIs of verifying keys to EOSIO/Antelope names.
#### type
`eosio::multi_index`
#### struct
```
id: eosio::name
vk: std::string
```

* * *
### notes
Represents [Transmitted UTXO Ciphertext List](protocol/datasets.md#transmitted-utxo-ciphertext-list) which holds all UTXO ciphertexts that are transmitted via [In-band secret distribution of UTXOs](protocol/in-band.md).
#### type
`dapp::advanced_multi_index`
#### struct
```
id: uint64_t
block_number: uint64_t
leaf_index: uint64_t
encrypted_note: TransmittedNoteCiphertext
```

### mt

### nullifiers

### roots

### txbuffer

### assetbuffer

### global

## Actions

### init

### setvk

### begin

### step

### exec

### ontransfer