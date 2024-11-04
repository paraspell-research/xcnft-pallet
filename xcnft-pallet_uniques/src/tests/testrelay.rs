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

//! Relay chain runtime mock.

mod xcm_config;
pub use xcm_config::*;

use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{
		AsEnsureOriginWithArg, ConstU128, Everything, Nothing, ProcessMessage, ProcessMessageError,
	},
	weights::{Weight, WeightMeter},
};

use frame_system::{EnsureRoot,EnsureSigned};
use sp_core::ConstU32;
use sp_runtime::{traits::{Verify,IdentityLookup}, AccountId32,MultiSignature};
use polkadot_runtime_parachains::{
	configuration,
	inclusion::{AggregateMessageOrigin, UmpQueueId},
	origin, shared,
};
use xcm::latest::prelude::*;
use xcm_builder::{IsConcrete, SignedToAccountId32};
use xcm_executor::XcmExecutor;

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

impl shared::Config for Runtime {
	type DisabledValidators = ();
}

impl configuration::Config for Runtime {
	type WeightInfo = configuration::TestWeightInfo;
}

pub type LocalOriginToLocation =
	SignedToAccountId32<RuntimeOrigin, AccountId, constants::RelayNetwork>;

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	// Anyone can execute XCM messages locally...
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Everything;
	type XcmReserveTransferFilter = Everything;
	type Weigher = weigher::Weigher;
	type UniversalLocation = constants::UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = IsConcrete<constants::TokenLocation>;
	type TrustedLockers = ();
	type SovereignAccountOf = location_converter::LocationConverter;
	type MaxLockers = ConstU32<8>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type AdminOrigin = EnsureRoot<AccountId>;
}

impl origin::Config for Runtime {}

type Block = frame_system::mocking::MockBlock<Runtime>;

parameter_types! {
	/// Amount of weight that can be spent per block to service messages.
	pub MessageQueueServiceWeight: Weight = Weight::from_parts(1_000_000_000, 1_000_000);
	pub const MessageQueueHeapSize: u32 = 65_536;
	pub const MessageQueueMaxStale: u32 = 16;
}

/// Message processor to handle any messages that were enqueued into the `MessageQueue` pallet.
pub struct MessageProcessor;
impl ProcessMessage for MessageProcessor {
	type Origin = AggregateMessageOrigin;

	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut WeightMeter,
		id: &mut [u8; 32],
	) -> Result<bool, ProcessMessageError> {
		let para = match origin {
			AggregateMessageOrigin::Ump(UmpQueueId::Para(para)) => para,
		};
		xcm_builder::ProcessXcmMessage::<
			Junction,
			xcm_executor::XcmExecutor<XcmConfig>,
			RuntimeCall,
		>::process_message(message, Junction::Parachain(para.into()), meter, id)
	}
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Size = u32;
	type HeapSize = MessageQueueHeapSize;
	type MaxStale = MessageQueueMaxStale;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = ();
	type MessageProcessor = MessageProcessor;
	type QueueChangeHandler = ();
	type QueuePausedQuery = ();
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Balances: pallet_balances,
		ParasOrigin: origin,
		XcmPallet: pallet_xcm,
		NFTs: pallet_uniques,
		MessageQueue: pallet_message_queue,
	}
);