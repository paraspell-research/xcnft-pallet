#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use codec::Encode;
	use cumulus_pallet_xcm::{ensure_sibling_para, Origin as CumulusOrigin};
	use cumulus_primitives_core::ParaId;
	use frame_support::{
		pallet_prelude::*,
		parameter_types,
		sp_runtime::traits::Hash,
		traits::{Currency, LockableCurrency, ReservableCurrency},
		BoundedVec,
	};
	use frame_system::{
		pallet_prelude::{OriginFor, *},
		Config as SystemConfig,
	};
	use scale_info::prelude::vec;
	use sp_std::prelude::*;
	use xcm::latest::prelude::*;

	/// Used to limit maximal string and json size
	pub type BoundedString<T> = BoundedVec<u8, <T as Config>::StringLimit>;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type ParachainID = ParaId;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		///Get string and json limits from defined values
		type StringLimit: Get<u32>;
		type JsonLimit: Get<u32>;
		///Get max collection size limit
		type CollectionLimit: Get<u32>;
		///Get max parachain ID limit
		type ParaIDLimit: Get<u32>;

		///Get limit of collections per Parachain
		type CollectionsPerParachainLimit: Get<u32>;

		//Get limit of nfts per Parachain
		type NFTsPerParachainLimit: Get<u32>;

		/// Type to access the Balances Pallet.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;

		type RuntimeOrigin: From<<Self as SystemConfig>::RuntimeOrigin>
			+ Into<Result<CumulusOrigin, <Self as Config>::RuntimeOrigin>>;

		/// The overarching call type; we assume sibling chains use the same type.
		type RuntimeCall: From<Call<Self>> + Encode;

		type XcmSender: SendXcm;
	}

	// Storage getters for collections and non-fungibles created by mint function
	#[pallet::storage]
	#[pallet::getter(fn collections)]
	pub type Collections<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, CollectionWithHash<T>>;

	#[pallet::storage]
	#[pallet::getter(fn non_fungibles)]
	pub type NonFungibles<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, NonFungible<T>>;

	//We will have storage holding collections and non-fungibles available on other chains
	#[pallet::storage]
	#[pallet::getter(fn other_chains_cols)]
	pub type OtherChainCollections<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ParachainID,
		BoundedVec<CollectionWithHash<T>, T::CollectionsPerParachainLimit>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn other_chains_nfs)]
	pub type OtherChainNonFungibles<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ParachainID,
		BoundedVec<NonFungible<T>, T::NFTsPerParachainLimit>,
	>;

	//We will have storage for collections received from other chains
	#[pallet::storage]
	#[pallet::getter(fn received_cols)]
	pub type ReceivedCollections<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ParachainID,
		BoundedVec<CollectionWithHash<T>, T::CollectionsPerParachainLimit>,
	>;

	//We will have storage for non-fungibles received from other chains
	#[pallet::storage]
	#[pallet::getter(fn received_nfts)]
	pub type ReceivedNonFungibles<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ParachainID,
		BoundedVec<NonFungible<T>, T::NFTsPerParachainLimit>,
	>;

	//We will save collection size to storage
	#[pallet::storage]
	#[pallet::getter(fn collection_size)]
	pub type CollectionSize<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, u32>;

	// Lets define our structures for collections and non-fungibles
	#[derive(Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Collection<T: Config> {
		pub owner: T::AccountId,
		pub collection_name: BoundedString<T>,
		pub collection_description: BoundedString<T>,
		pub collection_origin_parachain_id: ParachainID,
	}

	// Lets define our structures for collections and non-fungibles
	#[derive(Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct CollectionWithHash<T: Config> {
		pub owner: T::AccountId,
		pub collection_name: BoundedString<T>,
		pub collection_description: BoundedString<T>,
		pub collection_origin_parachain_id: ParachainID,
		pub collection_hash: T::Hash,
	}

	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct NonFungible<T: Config> {
		pub owner: T::AccountId,
		pub collection_hash: T::Hash,
		pub nft_name: BoundedString<T>,
		pub nft_description: BoundedString<T>,
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		MintedNonFungibleTransfered {
			nft_hash: T::Hash,
			owner: T::AccountId,
			destination: ParaId,
		},
		NonFungibleTransfered {
			nft_hash: T::Hash,
			owner: T::AccountId,
			destination: ParaId,
		},
		CollectionCreatedAndTransferedXCM {
			collection_hash: T::Hash,
			owner: T::AccountId,
			destination: ParaId,
		},
		CollectionTransfered {
			collection_hash: T::Hash,
			owner: T::AccountId,
			destination: ParaId,
		},
		CollectionReceived {
			collection_hash: T::Hash,
			owner: T::AccountId,
			origin: ParaId,
		},
		CollectionMinted {
			collection_hash: T::Hash,
			owner: T::AccountId,
		},
		NonFungibleMinted {
			nft_hash: T::Hash,
			owner: T::AccountId,
		},
		TokensDeposited {
			who: T::AccountId,
			amount: BalanceOf<T>,
		},
		CollectionFailedToXCM {
			e: SendError,
			collection_hash: T::Hash,
			owner: T::AccountId,
			destination: ParaId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		Unauthorized,       //Used when user wants to mint into collection they do not own
		InvalidNonFungible, //Used when user provides invalid NFT hash
		InvalidCollection,  //Used when user provides invalid collection number
		CollectionAlreadyExists, /* Used when user tries to create or transfer a collection
		                     * that already exists */
		CollectionAlreadyExistsOnOtherChain, /* Used when user tries to create or transfer a
		                                      * collection
		                                      * that already exists on other chain */
		CollectionWasNotAddedToOtherChain, //Error that should never happen
		CollectionWasNotAdded,             //Error that should never happen
		CollectionFull,                    /* Used when user tries to add new non-fungible but
		                                    * collection is full */
		InvalidDestination,       //Parachain ID does not exist or is above limit
		InvalidAccount,           //Account provided is not correct
		InsufficientBalance,      //Insufficient balance to send XCM
		CollectionWasNotReceived, //If collection exists perhaps, then do not add it again
		NonFungibleAlreadyExists,
		CollectionIsNotSentCrossChain, /* Used when user tries to mint non fungible from other
		                                * chain */
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/*
		TBD (ADD XCM): ADD XCM -- Function creates collection and sends it cross-chain
		*/

		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn mint_collection_xtransfer(
			origin: OriginFor<T>,
			collection_name: BoundedString<T>,
			collection_description: BoundedString<T>,
			destination: ParaId,
			recipient: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			//Check if paraID is within limit
			//ensure!(destination <= T::ParaIDLimit::get(), Error::<T>::InvalidDestination);

			//create collection, fetch parachain ID
			let collection: Collection<T> = Collection {
				owner: recipient.clone(),
				collection_name: collection_name.clone(),
				collection_description: collection_description.clone(),
				collection_origin_parachain_id: 2000.into(),
			};

			//create hash of collection
			let collection_hash = T::Hashing::hash_of(&collection);

			let collection_with_hash = CollectionWithHash {
				owner: recipient.clone(),
				collection_name: collection_name.clone(),
				collection_description: collection_description.clone(),
				collection_origin_parachain_id: 2000.into(),
				collection_hash: collection_hash.clone(),
			};

			//create copy of collection
			let collection_copy = collection_with_hash.clone();

			let xcol = OtherChainCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();
			let xcolec = ReceivedCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();

			//Now iterate through vector and check
			for x in xcol {
				for y in x {
					ensure!(y != collection_with_hash, Error::<T>::CollectionAlreadyExists);
				}
			}

			//Do same for received collections
			for x in xcolec {
				for y in x {
					ensure!(y != collection_with_hash, Error::<T>::CollectionAlreadyExists);
				}
			}

			ensure!(
				!Collections::<T>::contains_key(collection_hash),
				Error::<T>::CollectionAlreadyExists
			);

			//set collection size to 0 in storage
			let _ = CollectionSize::<T>::insert(collection_hash, 0);

			//Allocate fee for XCM transfer & collection creation
			let user_ballance = T::Currency::free_balance(&who);

			//If collection does not yet exist create it and insert it into other chain collections
			let _ = OtherChainCollections::<T>::mutate(destination, |x| -> Result<(), ()> {
				if let Some(x) = x {
					x.try_push(collection_with_hash).map_err(|_| ())?;
					Ok(())
				} else {
					*x = Some(vec![collection_with_hash].try_into().map_err(|_| ())?);
					Ok(())
				}
			});

			//We check if collection was added to other chain collections
			ensure!(
				OtherChainCollections::<T>::get(destination)
					.unwrap_or_default()
					.contains(&collection_copy),
				Error::<T>::CollectionWasNotAddedToOtherChain
			);

			//let para = ensure_sibling_para(<T as Config>::RuntimeOrigin::from(destination.clone()))?;

			match send_xcm::<T::XcmSender>(
				(Parent, Junction::Parachain(destination.into())).into(),
				Xcm(vec![Transact {
					origin_kind: OriginKind::Native,
					require_weight_at_most: Weight::from_parts(1_000, 1_000),
					call: <T as Config>::RuntimeCall::from(Call::<T>::mint_collection_received {
						collection_name,
						collection_description,
						origin_parachain_id: 2000.into(),
						owner: recipient.clone(),
					})
					.encode()
					.into(),
				}]),
			) {
				Ok((hash, cost)) => Self::deposit_event(Event::CollectionCreatedAndTransferedXCM {
					collection_hash,
					owner: recipient.clone(),
					destination,
				}),
				Err(e) => Self::deposit_event(Event::CollectionFailedToXCM {
					e,
					collection_hash,
					owner: recipient.clone(),
					destination,
				}),
			}
			Ok(().into())
		}

		/*
		TBT: This function will serve for minting collections on other chains
		*/
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn mint_collection_received(
			origin: OriginFor<T>,
			collection_name: BoundedString<T>,
			collection_description: BoundedString<T>,
			origin_parachain_id: ParaId,
			owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//We check if collection already exists
			let collection: Collection<T> = Collection {
				owner: owner.clone(),
				collection_name: collection_name.clone(),
				collection_description: collection_description.clone(),
				collection_origin_parachain_id: origin_parachain_id,
			};

			let collection_hash = T::Hashing::hash_of(&collection);

			let collection_with_hash = CollectionWithHash {
				owner: owner.clone(),
				collection_name: collection_name.clone(),
				collection_description: collection_description.clone(),
				collection_origin_parachain_id: origin_parachain_id.clone(),
				collection_hash: collection_hash.clone(),
			};

			let collection_copy = collection_with_hash.clone();

			//Make sure that collection does not exist in received collections
			let xcol = OtherChainCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();
			let xcolec = ReceivedCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();

			//Now iterate through vector and check
			for x in xcol {
				for y in x {
					ensure!(y != collection_with_hash, Error::<T>::CollectionAlreadyExists);
				}
			}

			//Do same for received collections
			for x in xcolec {
				for y in x {
					ensure!(y != collection_with_hash, Error::<T>::CollectionAlreadyExists);
				}
			}

			ensure!(
				!Collections::<T>::contains_key(collection_hash),
				Error::<T>::CollectionAlreadyExists
			);

			//Add collection number of nfts to collection size
			let _ = CollectionSize::<T>::insert(collection_hash, 0);

			//Add collection to received collections
			let _ = ReceivedCollections::<T>::mutate(origin_parachain_id, |x| -> Result<(), ()> {
				if let Some(x) = x {
					x.try_push(collection_with_hash).map_err(|_| ())?;
					Ok(())
				} else {
					*x = Some(vec![collection_with_hash].try_into().map_err(|_| ())?);
					Ok(())
				}
			});

			//We check if collection was added to received collections
			ensure!(
				ReceivedCollections::<T>::get(origin_parachain_id)
					.unwrap_or_default()
					.contains(&collection_copy),
				Error::<T>::CollectionWasNotReceived
			);

			Self::deposit_event(Event::CollectionReceived {
				collection_hash,
				owner: owner,
				origin: origin_parachain_id,
			});
			Ok(().into())
		}
		/* 
		/*
		TBT: This function will serve for minting collections on same chain
		*/
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn mint_collection(
			origin: OriginFor<T>,
			collection_name: BoundedString<T>,
			collection_description: BoundedString<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//We check if collection already exists
			let collection: Collection<T> = Collection {
				owner: who.clone(),
				collection_name: collection_name.clone(),
				collection_description: collection_description.clone(),
				collection_origin_parachain_id: T::ParachainID::get(),
			};

			let collection_hash = T::Hashing::hash_of(&collection);

			let collection_with_hash = CollectionWithHash {
				owner: who.clone(),
				collection_name: collection_name.clone(),
				collection_description: collection_description.clone(),
				collection_origin_parachain_id: T::ParachainID::get(),
				collection_hash: collection_hash.clone(),
			};

			//Make sure that collection does not exist in collections
			let xcol = OtherChainCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();
			let xcolec = ReceivedCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();

			//Now iterate through vector and check
			for x in xcol {
				for y in x {
					ensure!(y != collection_with_hash, Error::<T>::CollectionAlreadyExists);
				}
			}

			//Do same for received collections
			for x in xcolec {
				for y in x {
					ensure!(y != collection_with_hash, Error::<T>::CollectionAlreadyExists);
				}
			}

			ensure!(
				!Collections::<T>::contains_key(collection_hash),
				Error::<T>::CollectionAlreadyExists
			);

			//Add collection number of nfts to collection size
			let _ = CollectionSize::<T>::insert(collection_hash, 0);

			//Add collection to collections
			Collections::<T>::insert(collection_hash, collection_with_hash);

			//We check if collection was added to collections
			ensure!(
				Collections::<T>::contains_key(collection_hash),
				Error::<T>::CollectionWasNotAdded
			);

			Self::deposit_event(Event::CollectionMinted { collection_hash, owner: who });

			//We check if collection already exists
			Ok(().into())
		}

		/*
		TBD (Add XCM): This function sends existing collection cross-chain
		*/
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn collection_xtransfer(
			origin: OriginFor<T>,
			collection_hash: T::Hash,
			destination: ParaId,
			owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//Make sure that the collection exists
			ensure!(Collections::<T>::contains_key(collection_hash), Error::<T>::InvalidCollection);

			//Make sure that the user owns the collection
			let collection = Collections::<T>::get(collection_hash).unwrap();
			ensure!(collection.owner == who, Error::<T>::Unauthorized);

			//Make sure that the collection does not exist in other chain collections
			let xcol = OtherChainCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();
			let xcolec = ReceivedCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();

			//Now iterate through vector and check
			for x in xcol {
				for y in x {
					ensure!(
						y.collection_hash != collection_hash,
						Error::<T>::CollectionAlreadyExists
					);
				}
			}

			//Do same for received collections
			for x in xcolec {
				for y in x {
					ensure!(
						y.collection_hash != collection_hash,
						Error::<T>::CollectionAlreadyExists
					);
				}
			}

			//Make sure destination is valid
			//ensure!(destination <= T::ParaIDLimit::get(), Error::<T>::InvalidDestination);

			//Get collection from collections
			let collection_with_hash = Collections::<T>::get(collection_hash).unwrap();

			//Remove collection from collections
			let _ = Collections::<T>::remove(collection_hash);

			//create copy of collection
			let collection_copy = collection_with_hash.clone();

			//Put collection to other chain collections
			let _ = OtherChainCollections::<T>::mutate(destination, |x| -> Result<(), ()> {
				if let Some(x) = x {
					x.try_push(collection_with_hash).map_err(|_| ())?;
					Ok(())
				} else {
					*x = Some(vec![collection_with_hash].try_into().map_err(|_| ())?);
					Ok(())
				}
			});

			//Check if collection was added to other chain collections successfuly
			ensure!(
				OtherChainCollections::<T>::get(destination)
					.unwrap_or_default()
					.contains(&collection_copy),
				Error::<T>::CollectionWasNotAddedToOtherChain
			);

			//THERE WILL BE XCM HERE

			Self::deposit_event(Event::CollectionTransfered {
				collection_hash,
				owner: owner,
				destination,
			});

			Ok(().into())
		}

		/*
		TBD (ADD XCM): This function creates and transfers non fungible cross-chain
		*/

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn mint_non_fungible_xtransfer(
			origin: OriginFor<T>,
			nft_name: BoundedString<T>,
			nft_description: BoundedString<T>,
			collection_hash: T::Hash,
			destination: ParaId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//Make sure collection exists in other chain collections under same parachain ID
			ensure!(
				OtherChainCollections::<T>::get(destination)
					.unwrap_or_default()
					.iter()
					.any(|x| x.collection_hash == collection_hash),
				Error::<T>::InvalidCollection
			);

			//retrieve collection
			let collection = OtherChainCollections::<T>::get(destination).unwrap_or_default();
			let coll = collection.iter().find(|x| x.collection_hash == collection_hash).unwrap();

			//Make sure that the user owns the collection
			ensure!(coll.owner == who, Error::<T>::Unauthorized);

			//Make sure that the collection is not full
			ensure!(
				CollectionSize::<T>::get(collection_hash).unwrap_or_default() <
					T::CollectionLimit::get(),
				Error::<T>::CollectionFull
			);

			//Make sure destination is valid
			//ensure!(destination <= T::ParaIDLimit::get(), Error::<T>::InvalidDestination);

			//Lets create nft
			let nft = NonFungible {
				owner: who.clone(),
				collection_hash: collection_hash.clone(),
				nft_name: nft_name.clone(),
				nft_description: nft_description.clone(),
			};

			let nft_hash = T::Hashing::hash_of(&nft);

			//Make sure there is no nft with the same hash in other chain non fungibles with same
			// parachain ID
			ensure!(
				!OtherChainNonFungibles::<T>::get(destination).unwrap_or_default().contains(&nft),
				Error::<T>::NonFungibleAlreadyExists
			);

			//IF NFT does not exist in other chain non fungibles, then check if it exists in
			// received non fungibles
			ensure!(
				!ReceivedNonFungibles::<T>::get(destination).unwrap_or_default().contains(&nft),
				Error::<T>::NonFungibleAlreadyExists
			);

			//Do same for non fungibles
			ensure!(
				!NonFungibles::<T>::contains_key(nft_hash),
				Error::<T>::NonFungibleAlreadyExists
			);

			//Otherwise insert nft into nfts
			let _ = OtherChainNonFungibles::<T>::mutate(destination, |x| -> Result<(), ()> {
				if let Some(x) = x {
					x.try_push(nft).map_err(|_| ())?;
					Ok(())
				} else {
					*x = Some(vec![nft].try_into().map_err(|_| ())?);
					Ok(())
				}
			});

			//Update collection size
			let mut collection_size = CollectionSize::<T>::get(collection_hash).unwrap();
			collection_size += 1;
			let _ = CollectionSize::<T>::insert(collection_hash, collection_size);

			//ADD XCM HERE

			Self::deposit_event(Event::MintedNonFungibleTransfered {
				nft_hash,
				owner: who,
				destination,
			});

			Ok(().into())
		}

		/*
		TBTested: This function creates non fungible
		*/

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn mint_non_fungible(
			origin: OriginFor<T>,
			nft_name: BoundedString<T>,
			nft_description: BoundedString<T>,
			collection_hash: T::Hash,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//Make sure that the collection exists
			ensure!(Collections::<T>::contains_key(collection_hash), Error::<T>::InvalidCollection);

			let collection = Collections::<T>::get(collection_hash).unwrap();
			//Make sure that the user owns the collection
			ensure!(collection.owner == who, Error::<T>::Unauthorized);

			//Make sure that the collection is not full
			ensure!(
				CollectionSize::<T>::get(collection_hash).unwrap_or_default() <
					T::CollectionLimit::get(),
				Error::<T>::CollectionFull
			);

			//Lets create nft
			let nft = NonFungible {
				owner: who.clone(),
				collection_hash: collection_hash.clone(),
				nft_name: nft_name.clone(),
				nft_description: nft_description.clone(),
			};

			let nft_hash = T::Hashing::hash_of(&nft);

			//Make sure there is no nft with the same hash
			let xcol = OtherChainNonFungibles::<T>::iter().map(|x| x.1).collect::<Vec<_>>();
			let xcolec = ReceivedNonFungibles::<T>::iter().map(|x| x.1).collect::<Vec<_>>();

			//Now iterate through vector and check
			for x in xcol {
				for y in x {
					ensure!(y != nft, Error::<T>::NonFungibleAlreadyExists);
				}
			}

			//Do same for received collections
			for x in xcolec {
				for y in x {
					ensure!(y != nft, Error::<T>::NonFungibleAlreadyExists);
				}
			}

			ensure!(
				!NonFungibles::<T>::contains_key(nft_hash),
				Error::<T>::NonFungibleAlreadyExists
			);

			NonFungibles::<T>::insert(nft_hash, nft);

			//Update collection size
			let mut collection_size = CollectionSize::<T>::get(collection_hash).unwrap();
			collection_size += 1;
			let _ = CollectionSize::<T>::insert(collection_hash, collection_size);

			Self::deposit_event(Event::NonFungibleMinted { nft_hash, owner: who });

			Ok(().into())
		}

		/*
		TBTested: This function mints non-fungible on receiving chain
		*/

		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn mint_non_fungible_received(
			origin: OriginFor<T>,
			nft_name: BoundedString<T>,
			nft_description: BoundedString<T>,
			collection_hash: T::Hash,
			destination: ParaId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut collections = ReceivedCollections::<T>::get(destination).unwrap_or_default();

			//Go through collections, if collection corresponds to collections_hash, then we found
			// the collection, set it to new mutable variable, otherwise return error
			let collection = collections
				.iter_mut()
				.find(|collection| collection.collection_hash == collection_hash)
				.ok_or(Error::<T>::InvalidCollection)?;

			//Make sure that the user owns the collection
			ensure!(collection.owner == who, Error::<T>::Unauthorized);

			//Make sure that the collection is not full
			ensure!(
				CollectionSize::<T>::get(collection_hash).unwrap_or_default() <
					T::CollectionLimit::get(),
				Error::<T>::CollectionFull
			);

			//Make sure destination is valid
			//ensure!(destination <= T::ParaIDLimit::get(), Error::<T>::InvalidDestination);

			//Lets create nft
			let nft = NonFungible {
				owner: who.clone(),
				collection_hash: collection_hash.clone(),
				nft_name: nft_name.clone(),
				nft_description: nft_description.clone(),
			};

			let nft_hash = T::Hashing::hash_of(&nft);

			//Make sure there isnt same nft
			let xcol = OtherChainNonFungibles::<T>::iter().map(|x| x.1).collect::<Vec<_>>();
			let xcolec = ReceivedNonFungibles::<T>::iter().map(|x| x.1).collect::<Vec<_>>();

			//Now iterate through vector and check
			for x in xcol {
				for y in x {
					ensure!(y != nft, Error::<T>::NonFungibleAlreadyExists);
				}
			}

			//Do same for received collections
			for x in xcolec {
				for y in x {
					ensure!(y != nft, Error::<T>::NonFungibleAlreadyExists);
				}
			}

			//Check if non fungible is not minted
			ensure!(
				!NonFungibles::<T>::contains_key(nft_hash),
				Error::<T>::NonFungibleAlreadyExists
			);

			//Otherwise insert nft into nfts
			let _ = ReceivedNonFungibles::<T>::mutate(destination, |x| -> Result<(), ()> {
				if let Some(x) = x {
					x.try_push(nft).map_err(|_| ())?;
					Ok(())
				} else {
					*x = Some(vec![nft].try_into().map_err(|_| ())?);
					Ok(())
				}
			});

			//increase collection size
			let mut collection_size = CollectionSize::<T>::get(collection_hash).unwrap();
			collection_size += 1;
			let _ = CollectionSize::<T>::insert(collection_hash, collection_size);

			Self::deposit_event(Event::NonFungibleMinted { nft_hash, owner: who });

			Ok(().into())
		}

		/*
		TBD(ADD XCM): This function will send existing non-fungibles cross-chain
		*/
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn non_fungible_xtransfer(
			origin: OriginFor<T>,
			nft_hash: T::Hash,
			destination: ParaId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//Make sure that the nft exists
			ensure!(NonFungibles::<T>::contains_key(nft_hash), Error::<T>::InvalidNonFungible);

			//Make sure that the user owns the nft
			let nft = NonFungibles::<T>::get(nft_hash).unwrap();
			ensure!(nft.owner == who, Error::<T>::Unauthorized);

			//Make clone of nft
			let nft_copy = nft.clone();

			//Make sure, that collection NFT belongs to is already on other chain
			let xcol = OtherChainCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();
			let xcolec = ReceivedCollections::<T>::iter().map(|x| x.1).collect::<Vec<_>>();

			//Now iterate through vector and check
			let mut check = 0;

			for x in xcol {
				for y in x {
					if (y.collection_hash == nft_copy.collection_hash) {
						check = 1;
						break;
					}
				}
			}

			ensure!(check == 1, Error::<T>::CollectionIsNotSentCrossChain);

			//Do same for received collections
			for x in xcolec {
				for y in x {
					ensure!(
						y.collection_hash != nft_copy.collection_hash,
						Error::<T>::CollectionIsNotSentCrossChain
					);
				}
			}

			//Also make sure collections does not contain collection
			ensure!(
				!Collections::<T>::contains_key(nft_copy.collection_hash),
				Error::<T>::CollectionIsNotSentCrossChain
			);

			//Make sure destination is valid
			//ensure!(destination <= T::ParaIDLimit::get(), Error::<T>::InvalidDestination);

			//Get nft from non fungibles
			let nft = NonFungibles::<T>::get(nft_hash).unwrap();

			//Remove nft from non fungibles
			let _ = NonFungibles::<T>::remove(nft_hash);

			//Put nft to other chain non fungibles
			let _ = OtherChainNonFungibles::<T>::mutate(destination, |x| -> Result<(), ()> {
				if let Some(x) = x {
					x.try_push(nft).map_err(|_| ())?;
					Ok(())
				} else {
					*x = Some(vec![nft].try_into().map_err(|_| ())?);
					Ok(())
				}
			});

			//THERE WILL BE XCM HERE

			//Check if nft was added to other chain non fungibles
			ensure!(
				OtherChainNonFungibles::<T>::get(destination)
					.unwrap_or_default()
					.contains(&nft_copy),
				Error::<T>::CollectionWasNotAddedToOtherChain
			);

			Self::deposit_event(Event::NonFungibleTransfered { nft_hash, owner: who, destination });

			Ok(().into())
		}

		/*
		DONE: Function to allow sudo add balance // FOR TESTING PURPOSE ONLY
		*/
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn deposit_token(
			origin: OriginFor<T>,
			who: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;

			//We deposit test tokens
			T::Currency::deposit_creating(&who, amount);

			//We emit an event about succesful deposit
			Self::deposit_event(Event::TokensDeposited { who, amount });

			Ok(())
		}*/
	}
}
