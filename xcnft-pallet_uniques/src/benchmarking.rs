//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::v2::*;
use frame_support::{assert_ok, traits::Currency, BoundedVec};
use frame_system::RawOrigin;
use pallet_uniques::BenchmarkHelper;
use sp_runtime::traits::{Bounded, StaticLookup};
use sp_std::vec;
type DepositBalanceOf<T, I> = <<T as pallet_uniques::Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[instance_benchmarks]
mod benchmarks {
	use super::*;

	// Benchmark tries the first scenario of collection_x_transfer (transfering empty collection)
	#[benchmark]
	fn transfer_collection_empty<T: Config<I>, I: 'static>() {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let collection = T::Helper::collection(0);

		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());

		assert_ok!(pallet_uniques::Pallet::<T, I>::create(
			RawOrigin::Signed(caller.clone()).into(),
			collection.clone(),
			caller_lookup.clone()
		));

		#[extrinsic_call]
		collection_x_transfer(
			RawOrigin::Signed(caller.into()),
			collection.clone(),
			None,
			1000.into(),
			None,
		);
	}

	// Benchmark tries the second scenario of collection_x_transfer (transfering collection with
	// items same owner)
	#[benchmark]
	fn transfer_collection_same_owner<T: Config<I>, I: 'static>() {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let collection = T::Helper::collection(0);
		let item = T::Helper::item(0);

		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());

		assert_ok!(pallet_uniques::Pallet::<T, I>::create(
			RawOrigin::Signed(caller.clone()).into(),
			collection.clone(),
			caller_lookup.clone()
		));
		assert_ok!(pallet_uniques::Pallet::<T, I>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			collection.clone(),
			item.clone(),
			caller_lookup.clone()
		));

		#[extrinsic_call]
		collection_x_transfer(
			RawOrigin::Signed(caller.into()),
			collection.clone(),
			None,
			1000.into(),
			None,
		);
	}

	// Benchmark tries the third scenario of collection_x_transfer (transfering collection with
	// items different owners)
	#[benchmark]
	fn transfer_collection_other_owners<T: Config<I>, I: 'static>() {
		let caller: T::AccountId = whitelisted_caller();
		let caller2: T::AccountId = account("caller2", 1, 1);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let caller_lookup2 = T::Lookup::unlookup(caller2.clone());
		let collection = T::Helper::collection(0);
		let item = T::Helper::item(0);
		let item2 = T::Helper::item(1);

		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());

		assert_ok!(pallet_uniques::Pallet::<T, I>::create(
			RawOrigin::Signed(caller.clone()).into(),
			collection.clone(),
			caller_lookup.clone()
		));
		assert_ok!(pallet_uniques::Pallet::<T, I>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			collection.clone(),
			item.clone(),
			caller_lookup.clone()
		));
		assert_ok!(pallet_uniques::Pallet::<T, I>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			collection.clone(),
			item2.clone(),
			caller_lookup2.clone()
		));

		#[extrinsic_call]
		collection_x_transfer(
			RawOrigin::Signed(caller.into()),
			collection.clone(),
			None,
			1000.into(),
			None,
		);
	}

	//Benchmark tries nft transfer
	#[benchmark]
	fn transfer_nft<T: Config<I>, I: 'static>() {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let collection = T::Helper::collection(0);
		let item = T::Helper::item(0);

		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());

		assert_ok!(pallet_uniques::Pallet::<T, I>::create(
			RawOrigin::Signed(caller.clone()).into(),
			collection.clone(),
			caller_lookup.clone()
		));
		assert_ok!(pallet_uniques::Pallet::<T, I>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			collection.clone(),
			item.clone(),
			caller_lookup.clone()
		));

		#[extrinsic_call]
		nft_x_transfer(
			RawOrigin::Signed(caller.into()),
			collection.clone(),
			item.clone(),
			1000.into(),
			collection.clone(),
			item.clone(),
		);
	}

	//Benchmark tries collection empty parse
	#[benchmark]
	fn parse_empty_col<T: Config<I>, I: 'static>() {
		let caller: T::AccountId = whitelisted_caller();
		let collection = T::Helper::collection(0);

		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());

		#[extrinsic_call]
		parse_collection_empty(
			RawOrigin::Signed(caller.into()),
			collection.clone(),
			None,
			BoundedVec::new(),
			None,
		);
	}

	//Benchmark tries collection parse with items
	#[benchmark]
	fn parse_same_owner_col<T: Config<I>, I: 'static>() {
		let caller: T::AccountId = whitelisted_caller();
		let collection = T::Helper::collection(0);
		let item = T::Helper::item(0);
		let nfts = vec![(item.clone(), BoundedVec::new())];

		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());

		#[extrinsic_call]
		parse_collection_same_owner(
			RawOrigin::Signed(caller.into()),
			None,
			BoundedVec::new(),
			nfts.clone(),
			1000.into(),
			collection.clone(),
			None,
		);
	}

	//Benchmark tries collection parse with items and different owners
	#[benchmark]
	fn parse_diff_owner_col<T: Config<I>, I: 'static>() {
		let caller: T::AccountId = whitelisted_caller();
		let caller2: T::AccountId = account("caller2", 1, 1);
		let caller_lookup2 = T::Lookup::unlookup(caller2.clone());
		let collection = T::Helper::collection(0);
		let item = T::Helper::item(0);
		let nfts = vec![(item.clone(), caller_lookup2, BoundedVec::new())];

		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());

		#[extrinsic_call]
		parse_collection_diff_owners(
			RawOrigin::Signed(caller.into()),
			None,
			BoundedVec::new(),
			nfts.clone(),
			1000.into(),
			collection.clone(),
			None,
		);
	}

	//Benchmark tries collection parse item
	#[benchmark]
	fn parse_item<T: Config<I>, I: 'static>() {
		let caller: T::AccountId = whitelisted_caller();
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let collection = T::Helper::collection(0);
		let item = T::Helper::item(0);

		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T, I>::max_value());

		assert_ok!(pallet_uniques::Pallet::<T, I>::create(
			RawOrigin::Signed(caller.clone()).into(),
			collection.clone(),
			caller_lookup.clone()
		));

		#[extrinsic_call]
		parse_nft_transfer(
			RawOrigin::Signed(caller.into()),
			collection.clone(),
			item.clone(),
			BoundedVec::new(),
			collection.clone(),
			item.clone(),
			1000.into(),
		);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
