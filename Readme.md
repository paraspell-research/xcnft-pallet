# This repository holds xNFT Pallet for cross-chain non-fungible asset sharing

# Compile 
```cargo build```

# Add to runtime
```
/// Configure the pallet xnft in pallets/xnft.
impl pallet_parachain_xnft::Config for Runtime {
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ConstU32<255>;
	type JsonLimit = ConstU32<255>;
	type CollectionLimit = ConstU32<255>;
	type ParaIDLimit = ConstU32<9999>;
	type CollectionsPerParachainLimit = ConstU32<9999>;
	type NFTsPerParachainLimit = ConstU32<9999>;
	type XcmSender = XcmRouter;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
}
```

and to construct_runtime! macro:

```
XNFTPallet: pallet_parachain_xnft = 51,
```

also import it

```
/// Import pallet xnft
pub use pallet_parachain_xnft;
```