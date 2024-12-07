# Stand out in a world of replicas - be unique 👨‍🎨

Your choice to implement xcNFT into your Parachain means a lot to us! Feel free to provide feedback or suggestions in the form of an issue in this [GitHub repository](https://github.com/paraspell-research/xcnft-pallet)!

Don't you know what xcNFT has to offer yet? Head to [User guide](https://paraspell-research.github.io/xcnft-docs/user-guide/intro.html) section of docs for an overview.

**Let's get you started.**

## Implementing xcNFT 👨‍💻 
The following subsection guides you through the implementation phase of xcNFT

Here are the steps that you should follow:

1. Copy code from the [following folder](https://github.com/paraspell-research/xcnft-pallet/tree/main/xcnft-pallet_uniques) into your Parachain's pallets folder.

2. Setup dependencies in xcNFT's `cargo.toml` to latest versions and try to compile your Parachain's code.

## Runtime setup ⚙️

xcNFT requires minimal runtime config:
```
impl pallet_parachain_xcnft::Config for Runtime {
	type RuntimeEvent = RuntimeEvent; 
	type WeightInfo = pallet_parachain_xcnft::weights::SubstrateWeight<Runtime>;;
	type XcmSender = xcm_config::XcmRouter; 
	type RuntimeCall = RuntimeCall; 
	type ProposalTimeInBlocks = proposal_time_in_blocks_parameter; //How long should proposals for moving collections with different owners last? 100800 for approximately 2 weeks.
	type MaxOwners = max_owners_parameter; //How many owners can be in the collection? Should be limited to pallet_nfts configuration.
}
```

**IMPORTANT!**

For your chain to be compatible with the xcNFT of other chains, make sure to name the module in the same exact way as provided below:
```
#[runtime::pallet_index(INSERT_INDEX_HERE)]
pub type XcnftPallet = pallet_parachain_xcnft;
```

Also, do not forget to addthe  xcNFT pallet to `cargo.toml`:
```
pallet-parachain-xcnft = { VERSION HERE }

//Also include it into STD
"pallet-parachain-xcnft/std",

//Also into runtime benchmarks
pallet-parachain-xcnft/runtime-benchmarks,

//And try-runtime
pallet-parachain-xcnft/try-runtime,
```


## XCM Setup 🔬

The only tweak that you should do to your XCM config is to **enable aliasers**:
```
type Aliasers = Everything; //Only enable Everything in the testnet environment!
```
