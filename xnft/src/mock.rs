use crate as pallet_xnft;

use cumulus_primitives_core::ParaId;
use frame_support::{derive_impl, parameter_types, traits::Everything};

use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

use pallet_balances::AccountData;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		XnftModule: pallet_xnft,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const RegistryStringLimit: u32 = 255;
	pub const RegistryJsonLimit: u32 = 2048;
	pub const RegistryCollectionLimit: u32 = 255;
	pub const RegistryParaIDLimit: u32 = 9999;
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
	pub const MaxParachains: u32 = 100;
	pub const MaxPayloadSize: u32 = 1024;

	//Parachain ID - this has to be set by parachain team
	pub const RegistryParaId: ParaId = ParaId::new(2);

	//Max Amount of collections we will store per parachain
	pub const RegistryPerParachainCollectionLimit: u8 = 255;

	//Max Amount of NFTs we will store per parachain
	pub const RegistryNFTsPerParachainLimit: u32 = 255*255;
}
pub type XcmRouter = ();

impl pallet_xnft::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type StringLimit = RegistryStringLimit;
	type JsonLimit = RegistryJsonLimit;
	type CollectionLimit = RegistryCollectionLimit;
	type ParaIDLimit = RegistryParaIDLimit;
	type CollectionsPerParachainLimit = RegistryPerParachainCollectionLimit;
	type NFTsPerParachainLimit = RegistryNFTsPerParachainLimit;
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
