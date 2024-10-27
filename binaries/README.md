# Zombienet configs for every possible scenario ✨

The following folder contains binaries that can be used with Zombienet to start small networks. There are a total of 3 binaries:

- `config-both.toml`: This config launches testnet with 1 Relay chain, 1 Parachain of one kind and 1 Parachain of other kind

(Used in [this repository](https://github.com/paraspell-research/polkadot-sdk) to launch 1 Parachain with pallet_nfts and 1 Parachain with pallet_uniques xcNFT implementation.)

- `config-uniques.toml`: This config launches testnet with 1 Relay chain and 2 Parachains of the same kind

(Used in [this repository](https://github.com/paraspell-research/polkadot-sdk) to launch 2 Parachains with pallet_uniques xcNFT implementation.)

- `config-nfts.toml`: This config launches testnet with 1 Relay chain, 2 Parachain of the same kind same as config above

(Used in [this repository](https://github.com/paraspell-research/polkadot-sdk) to launch 2 Parachains with pallet_nfts xcNFT implementation.)
