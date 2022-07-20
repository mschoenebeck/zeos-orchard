# ZEOS Orchard

This is the main application for the [ZEOS](https://zeos.one) protocol for private and untraceable transactions on the EOS Blockchain. This application will be deployed to the [EOS Mainnet](https://eos.io/eos-public-blockchain/).

See also:
- [The ZEOS Book](https://mschoenebeck.github.io/zeos-orchard/) (including a full protocol specification)
- [Token Contract (Orchard)](https://github.com/mschoenebeck/thezeostoken/tree/orchard)
- [JS Wallet (Orchard)](https://github.com/mschoenebeck/zeos-wallet/tree/orchard)

## Description
This repository is a fork of [Zcash Orchard](https://github.com/zcash/orchard). The application enables Zcash-like shielded transactions of fungible and non-fungible tokens on the EOS blockchain. Check out the [Whitepaper](https://github.com/mschoenebeck/zeos-docs/releases/download/v1.0.0/zeos_whitepaper_v1.0.0.pdf) for more Information.

This application is built on [EOSIO](https://eos.io/) and [Liquidapps' DAPP Network](https://liquidapps.io/) services.

## Getting Started

To setup the full workspace clone the dependencies [rustzeos](https://github.com/mschoenebeck/rustzeos), [halo2](https://github.com/mschoenebeck/halo2), [pasta_curves](https://github.com/mschoenebeck/pasta_curves), [reddsa](https://github.com/mschoenebeck/reddsa), the smart contract and the JS wallet as well:

```
mkdir zeos
cd zeos
git clone https://github.com/mschoenebeck/rustzeos.git
git clone https://github.com/mschoenebeck/halo2.git
git clone https://github.com/mschoenebeck/pasta_curves.git
git clone https://github.com/mschoenebeck/reddsa.git
git clone https://github.com/mschoenebeck/thezeostoken.git
cd thezeostoken && git checkout orchard && cd ..
git clone https://github.com/mschoenebeck/zeos-wallet.git
cd zeos-wallet && git checkout orchard && cd ..
```

Clone this repository:

```
git clone https://github.com/mschoenebeck/zeos-orchard.git
cd zeos-orchard
```

Build the project as Rust library:

```
cargo build
```

### Dependencies

- [Rust Toolchain](https://www.rust-lang.org/tools/install)

## Help
If you need help join us on [Telegram](https://t.me/ZeosOnEos).

## Authors

Matthias Schönebeck

## License

It's open source. Do with it whatever you want.

## Acknowledgments

Big thanks to the Electric Coin Company for developing, documenting and maintaining this awesome open source codebase for zk-SNARKs!

* [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)