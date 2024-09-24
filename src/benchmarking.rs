#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Template;

use cumulus_primitives_core::ParaId;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::traits::Hash;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn mint_collection_large() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		mint_collection(RawOrigin::Signed(caller),b"pHgJCOTaAPedH0mtvRVVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(), b"pHgJCOTaAPEceLC69VVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap());
	}

	#[benchmark]
	fn mint_non_fungible_large() {
		let caller: T::AccountId = whitelisted_caller();

		let collection: pallet::Collection<T> = Collection {
			owner: caller.clone(),
			collection_name: b"pHgJCOTaAPedH0mtvRVVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),
			collection_description: b"pHgJCOTaAPEceLC69VVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),
			collection_origin_parachain_id: 0.into(),
		};

		let collection_hash = T::Hashing::hash_of(&collection);

		let collection_with_hash: pallet::CollectionWithHash<T> = CollectionWithHash {
			owner: caller.clone(),
			collection_name: b"pHgJCOTaAPedH0mtvRVVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),
			collection_description: b"pHgJCOTaAPEceLC69VVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),
			collection_origin_parachain_id: 0.into(),
			collection_hash: collection_hash,
		};

		pallet::Collections::<T>::insert(collection_hash, collection_with_hash);
		pallet::CollectionSize::<T>::insert(collection_hash, 0);

		#[extrinsic_call]
		mint_non_fungible(RawOrigin::Signed(caller),b"pHgJCOTaAPedH0mtvRVVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(), b"pHgJCOTaAPEceLC69VVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),collection_hash);
	}

	#[benchmark]
	fn mint_collection_received_large() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		mint_collection_received(RawOrigin::Signed(caller),b"pHgJCOTaAPedH0mtvRVVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(), b"pHgJCOTaAPEceLC69VVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),1000.into(), caller.clone());
	}

	#[benchmark]
	fn mint_non_fungible_received_large() {
		let caller: T::AccountId = whitelisted_caller();

		let collection: pallet::Collection<T> = Collection {
			owner: caller.clone(),
			collection_name: b"pHgJCOTaAPedH0mtvRVVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),
			collection_description: b"pHgJCOTaAPEceLC69VVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),
			collection_origin_parachain_id: 0.into(),
		};

		let collection_hash = T::Hashing::hash_of(&collection);

		let collection_with_hash: pallet::CollectionWithHash<T> = CollectionWithHash {
			owner: caller.clone(),
			collection_name: b"pHgJCOTaAPedH0mtvRVVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),
			collection_description: b"pHgJCOTaAPEceLC69VVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),
			collection_origin_parachain_id: 0.into(),
			collection_hash: collection_hash,
		};

		let parachain_id: ParaId = 0.into();
		let _ = ReceivedCollections::<T>::mutate(parachain_id, |x| -> Result<(), ()> {
			if let Some(x) = x {
				x.try_push(collection_with_hash).map_err(|_| ())?;
				Ok(())
			} else {
				*x = Some(vec![collection_with_hash].try_into().map_err(|_| ())?);
				Ok(())
			}
		});

		pallet::CollectionSize::<T>::insert(collection_hash, 0);

		#[extrinsic_call]
		mint_non_fungible_received(RawOrigin::Signed(caller),b"pHgJCOTaAPedH0mtvRVVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(), b"pHgJCOTaAPEceLC69VVtYyApzDcR0WebyTdJw1sSIRTozxXKFI3cA91Yv3ZzFk5ZH00J2SC7a3aFDrlt5rPIpwGO5UE6jSplFqdu7AhoEf8t7D6aD5CDBGOL8AZnllhAIBKBLgspdsSGoacIWx0CLFpPF2ALtm1iitrDo4B39sZC2ne9PGFIe4C1PazXLnWwbF0Kea3akaoNv6HRqKWgFNv4VTxdhCoWDsAzbzzP3GSyBYuSwBlhXP".to_vec().try_into().unwrap(),collection_hash, 0.into(), caller.clone());
	}

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
