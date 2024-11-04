use frame_support::{parameter_types, traits::VariantCountOf, weights::constants::RocksDbWeight};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned, GenesisConfig};
use pallet_balances::AccountData;
use sp_core::ConstU32;
use sp_runtime::{
	traits::{ConstU64, Verify},
	AccountId32 as AccountId, BuildStorage, MultiSignature,
};
use xcm_builder::WithUniqueTopic;

pub type XcmRouter = WithUniqueTopic<((),)>;

// Configure a mock runtime to test the pallet.
#[frame_support::runtime]
mod test_runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;

	#[runtime::pallet_index(5)]
	pub type XcNFT = crate;

	#[runtime::pallet_index(2)]
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(3)]
	pub type Uniques = pallet_uniques;

	#[runtime::pallet_index(4)]
	pub type ParachainInfo = parachain_info;
}

/// Balance of an account.
pub type Balance = u128;

impl frame_system::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = RuntimeTask;
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<AccountId>;
	type Nonce = u64;
	type Block = MockBlock<Test>;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = RocksDbWeight;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type ExtensionsWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

impl parachain_info::Config for Test {}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type XcmSender = XcmRouter;
	type RuntimeCall = RuntimeCall;
	type ProposalTimeInBlocks = proposal_time_in_blocks_parameter;
	type MaxOwners = max_owners_parameter;
}

pub const UNIT: Balance = 1;

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type DoneSlashHandler = ();
}

parameter_types! {
	pub const SS58Prefix: u16 = 42;
	pub const ExistentialDeposit: u128 = 500;
	pub const CollectionDeposit: Balance = 0 * UNIT; // 1 UNIT deposit to create asset collection
	pub const ItemDeposit: Balance = 0 * UNIT; // 1/100 UNIT deposit to create asset item
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 64;
	pub const UniquesMetadataDepositBase: Balance = 0 * UNIT;
	pub const AttributeDepositBase: Balance = 0 * UNIT;
	pub const DepositPerByte: Balance = 0 * UNIT;
	pub const UniquesStringLimit: u32 = 32;
	pub const ApprovalsLimit: u32 = 1;
	pub const ItemAttributesApprovalsLimit: u32 = 1;
	pub const MaxTips: u32 = 1;
	pub const MaxDeadlineDuration: u32 = 1;
	pub const MaxAttributesPerCall: u32 = 10;
	pub const proposal_time_in_blocks_parameter: u32 = 10;
	pub const max_owners_parameter: u32 = 1000000;
	pub const max_votes: u32 = 1000000;
}

pub type AccountPublic = <MultiSignature as Verify>::Signer;

impl pallet_uniques::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type CreateOrigin = EnsureSigned<AccountId>;
	type Locker = ();
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
