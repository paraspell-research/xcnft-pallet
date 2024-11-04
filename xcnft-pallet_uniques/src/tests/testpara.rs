// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Parachain runtime mock.

mod xcm_config;
pub use xcm_config::*;

use parachain_info;

use core::marker::PhantomData;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{ConstU128, ContainsPair, EnsureOrigin, EnsureOriginWithArg, Everything, Nothing},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
};
use frame_support::pallet_prelude::Get;
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::ConstU32;
use sp_runtime::{
	traits::{ConstU64, Verify, IdentityLookup},
	AccountId32, BuildStorage, MultiSignature,
};
use xcm::latest::prelude::*;
use xcm_builder::{EnsureXcmOrigin, SignedToAccountId32};
use xcm_executor::{traits::ConvertLocation, XcmExecutor};
use xcm_simulator::mock_message_queue;

pub type AccountId = AccountId32;
pub type Balance = u128;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
}


pub const UNIT: Balance = 1;
parameter_types! {
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

impl pallet_uniques::Config for Runtime {
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

// `EnsureOriginWithArg` impl for `CreateOrigin` which allows only XCM origins
// which are locations containing the class location.
pub struct ForeignCreators;
impl EnsureOriginWithArg<RuntimeOrigin, Location> for ForeignCreators {
	type Success = AccountId;

	fn try_origin(
		o: RuntimeOrigin,
		a: &Location,
	) -> core::result::Result<Self::Success, RuntimeOrigin> {
		let origin_location = pallet_xcm::EnsureXcm::<Everything>::try_origin(o.clone())?;
		if !a.starts_with(&origin_location) {
			return Err(o);
		}
		xcm_config::location_converter::LocationConverter::convert_location(&origin_location)
			.ok_or(o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(a: &Location) -> Result<RuntimeOrigin, ()> {
		Ok(pallet_xcm::Origin::Xcm(a.clone()).into())
	}
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
	pub const ReservedDmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
}

impl mock_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type LocalOriginToLocation =
	SignedToAccountId32<RuntimeOrigin, AccountId, constants::RelayNetwork>;

pub struct TrustedLockerCase<T>(PhantomData<T>);
impl<T: Get<(Location, AssetFilter)>> ContainsPair<Location, Asset> for TrustedLockerCase<T> {
	fn contains(origin: &Location, asset: &Asset) -> bool {
		let (o, a) = T::get();
		a.matches(asset) && &o == origin
	}
}

parameter_types! {
	pub RelayTokenForRelay: (Location, AssetFilter) = (Parent.into(), Wild(AllOf { id: AssetId(Parent.into()), fun: WildFungible }));
}

pub type TrustedLockers = TrustedLockerCase<RelayTokenForRelay>;

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = weigher::Weigher;
	type UniversalLocation = constants::UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = TrustedLockers;
	type SovereignAccountOf = location_converter::LocationConverter;
	type MaxLockers = ConstU32<8>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type AdminOrigin = EnsureRoot<AccountId>;
}

impl crate::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type XcmSender = XcmRouter;
	type RuntimeCall = RuntimeCall;
	type ProposalTimeInBlocks = proposal_time_in_blocks_parameter;
	type MaxOwners = max_owners_parameter;
}

type Block = frame_system::mocking::MockBlock<Runtime>;

impl parachain_info::Config for Runtime {}

construct_runtime!(
	pub struct Runtime {
		System: frame_system,
		Balances: pallet_balances,
		MsgQueue: mock_message_queue,
		PolkadotXcm: pallet_xcm,
        NFTs: pallet_uniques,
        XcNFT: crate,
        ParachainInfo: parachain_info,
	}
);