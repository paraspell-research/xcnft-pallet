# Welcome fellow Parachain developer 🧙‍♂️ - meet xcNFT

Following section will guide you through implementation of the xcNFT cross-chain pallet for non-fungible assets.

Pallet docs - [[link]](https://paraspell-research.github.io/xcnft-docs/)

## Before your journey begins 🧳

Before you begin with implementing this magical ✨ pallet into your Parachain, you must ensure, that you meet following pre-requirements:

### Your Parachain should implement following pallets:

**Substrate:**
 - `frame-benchmarking`
 - `frame-support`
 - `frame-system`

**Cumulus:**
 - `cumulus-primitives-core`
 - `cumulus-pallet-xcm`

**XCMP:**
 - `xcm`

**SP:**
 - `sp-runtime`
 - `sp-std`
 - `sp-core`
 - `sp-io`

**Substrate Pallets:**
 - Either `pallet-nfts` or `pallet-uniques` depending on which pallet your Parachain uses
 - `pallet-balances`
 - `parachain-info`
  
**Other pallets (Only needed for pallet_uniques xcnft version):**
 - `enumflags2`


### You should also make choice of xcnft version:
- If your Parachain uses **pallet_nfts** head over to [xcnft for pallet_nfts](https://paraspell-research.github.io/xcnft-docs/implementation-guide/pallet-nfts.html) section.
- If your Parachain uses **pallet_uniques** head over to [xcnft for pallet_uniques](https://paraspell-research.github.io/xcnft-docs/implementation-guide/pallet-uniques.html) section.

## Testing pallet functionality 🔎

Don't know whether this pallet is useful for your Parachain?

No worries! 

**Try it out before you implement it!**

### Dockerized local testnet build:
**Make sure your docker deamon is running**
Copy the [Dockerfile](https://github.com/paraspell-research/xcnft-pallet/blob/main/Dockerfile) from this repository and input following commands:
- `docker build --platform linux/x86_64 -t polkadot-sdk-image:latest .` to build Docker image
- `docker run --platform linux/x86_64 -p 9910-9913:9910-9913 -p 9920-9921:9920-9921 --rm -it polkadot-sdk-image:latest` to start Zombienet.

Once Zombienet is started, continue from step 8 in the next section.

### Follow these steps to create local testnet that implements xcNFT:

1. Fork or clone [following repository](https://github.com/paraspell-research/polkadot-sdk)

2. Download [Zombienet binary for your system](https://github.com/paritytech/zombienet/releases)

3. Copy zombienet binary into binaries folder of the repository you just forked

4. Compile Relay chain by: 
- `cd polkadot`
- `cargo build --release`

5. Compile first Parachain by:
- `cargo b -r -p parachain-template-node` 

6. Compile second Parachain by:
- `cargo b -r -p parachain-template-node-two`

7. Launch zombienet localhost network by:
- `cd binaries`
- Choose config that you wish to test:
    - `./zombienet-macos-arm64 -p native -c 1 spawn config-both.toml` launches network with 1 Relay chain, 1 Parachain with *pallet_nfts* and 1 Parachain with *pallet_uniques* (Best for testing pallet agnosticity)
    - `./zombienet-macos-arm64 -p native -c 1 spawn config-nfts.toml` launches network with 1 Relay chain and 2 Parachains with *pallet_nfts*
    - `./zombienet-macos-arm64 -p native -c 1 spawn config-uniques.toml` launches network with 1 Relay chain and 2 Parachains with *pallet_uniques*

8. Once the network is launched, connect to one of the Relay chain nodes:

![zombienet](https://github.com/user-attachments/assets/06f1d41e-55a7-4d7b-b7f3-f3e6fa276132)

9. Open HRMP channels for both chains (Needed for allowing cross-chain communication):
- Navigate to `Extrinsics` tab select `Alice` account and go to `Decode`
    - Paste following hash to Decode section: `0xff00fa05e8030000e90300000800000000040000` (This hash opens channel from chain 1000 to 1001)
    - Move back to Submission and sign the call with Alice
    - Paste following hash to Decode section `0xff00fa05e9030000e80300000800000000040000` (This hash opens channel from chain 1001 to 1000)
    - Move back to Submission and sign the call with Alice

10. Start interacting with chains by connecting to their WS endpoints from Zombienet console and try xcNFT out.

Don't know which function does what? Unsure what storage stores what? 

Head over to [User guide](https://paraspell-research.github.io/xcnft-docs/user-guide/intro.html) section.
 

## Other tests 🕹️

To test the pallet we constructed various unit tests with variations of data. To run these tests use command:

Choose pallet version and go to it's folder:
`cd xcnft-pallet_nfts || xcnft-pallet_uniques`

Run tests and benchmarks
`cargo test `
