//! # xcNFT Pallet by ParaSpell✨ Foundation team
//!
//! Pallet is made under [MIT](https://github.com/paraspell-research-foundation/xcNFT-Pallet/blob/main/LICENSE) license and is thereby free to use, modify, and distribute.
//!
//! A pallet that allows you to share your **Uniques** or **NFTs** across parachains.
//!
//!  As mentioned above, this pallet supports both Uniques and NFTs pallets for NFT management.
//!
//! ## Overview
//!
//! This pallet consists of following functionalities:
//! - Transferring empty collection cross-chain: **collectionXtransfer**
//! - Transferring non-empty collection cross-chain (All NFTs are owned by collection owner):
//!   **collectionXtransfer**
//! - Transfering non-empty collection cross-chain (NFTs are distributed between different
//!   accounts): **collectionXtransfer** & **collectionXtransferVote** &
//!   **collectionXtransferInitiate**
//! - Transfering non-fungible assets cross-chain: **nftXtransfer** & **nftXclaim**
//! - Updating collection metadata cross-chain: **collectionXupdate**
//! - Updating non-fungible asset metadata cross-chain: **nftXupdate**
//! - Burning collection cross-chain: **collectionXburn**
//! - Burning non-fungible asset cross-chain: **nftXburn**
//! - Transferring collection ownership cross-chain: **collectionXownership**
//! - Transferring non-fungible asset ownership cross-chain: **nftXownership**
//!
//! Each function within pallet has its own weight and is defined in `weights.rs` file.
//!
//! Each function is also annotated with comments explaining the purpose and functionality design of
//! the function.
//!
//! ## Dependencies
//! This pallet depends on the following pallets:
//!
//! Substrate:
//! - `frame-benchmarking`
//! - `frame-support`
//! - `frame-system`
//!
//! Cumulus:
//! - `cumulus-primitives-core`
//! - `cumulus-pallet-xcm`
//!
//! XCMP:
//! - `xcm`
//!
//! SP:
//! - `sp-runtime`
//! - `sp-std`
//! - `sp-core`
//! - `sp-io`
//!
//! Substrate Pallets:
//! - `pallet-nfts`
//! - `pallet-uniques`
//! - `pallet-balances`
//! - `parachain-info`

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use core::marker::PhantomData;
	use cumulus_primitives_core::ParaId;
	use frame_support::{
		dispatch::{DispatchResultWithPostInfo, PostDispatchInfo},
		pallet_prelude::*,
		traits::Incrementable,
	};
	use frame_system::pallet_prelude::*;
	use pallet_nfts::{CollectionConfigFor, DestroyWitness};
	use scale_info::prelude::vec;
	use sp_runtime::{traits::StaticLookup, DispatchError, DispatchErrorWithPostInfo};
	use sp_std::prelude::*;
	use xcm::latest::prelude::*;

	type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	#[pallet::config]
	pub trait Config<I: 'static = ()>:
		frame_system::Config + pallet_nfts::Config<I> + parachain_info::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching call type; we assume sibling chains use the same type.
		type RuntimeCall: From<Call<Self, I>> + Encode;

		/// The sender to use for cross-chain messages.
		type XcmSender: SendXcm;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: crate::weights::WeightInfo;

		/// Specifies how long should cross-chain proposals last
		type ProposalTimeInBlocks: Get<u32>;

		/// Specifies how manys different owners can be in a collection - used in voting process
		type MaxOwners: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	/// Enum for voting, either Aye or Nay option.
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Default, Debug)]
	#[scale_info(skip_type_params(T))]
	pub enum Vote {
		#[default]
		Aye,
		Nay,
	}

	/// Struct for votes, contains two vectors, one for Aye voters and one for Nay voters.
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Default, Debug)]
	#[scale_info(skip_type_params(T, I))]
	pub struct Votes<T: Config<I>, I: 'static = ()> {
		aye: BoundedVec<T::AccountId, T::MaxOwners>,
		nay: BoundedVec<T::AccountId, T::MaxOwners>,
	}

	/// Structure of proposal, contains proposal id, collection id, proposed collection owner,
	/// proposed destination parachain, proposed destination config, owners, number of votes, and
	/// end time.
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Default, Debug)]
	#[scale_info(skip_type_params(T, I))]
	pub struct Proposal<T: Config<I>, I: 'static = ()> {
		proposal_id: u64,
		collection_id: T::CollectionId,
		proposed_collection_owner: T::AccountId,
		proposed_destination_para: ParaId,
		proposed_destination_config: CollectionConfigFor<T, I>,
		owners: BoundedVec<T::AccountId, T::MaxOwners>,
		number_of_votes: Votes<T, I>,
		end_time: BlockNumberFor<T>,
	}

	/// Structure of sent assets, contains origin parachain id, origin collection id, origin asset
	/// id, destination collection id, and destination asset id.
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Default)]
	#[scale_info(skip_type_params(T, I))]
	pub struct SentStruct<T: Config<I>, I: 'static = ()> {
		origin_para_id: ParaId,
		origin_collection_id: T::CollectionId,
		origin_asset_id: T::ItemId,
		destination_collection_id: T::CollectionId,
		destination_asset_id: T::ItemId,
	}

	/// Structure of received assets, contains origin parachain id, origin collection id, origin
	/// asset id, received collection id, and received asset id.
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Default)]
	#[scale_info(skip_type_params(T, I))]
	pub struct ReceivedStruct<T: Config<I>, I: 'static = ()> {
		origin_para_id: ParaId,
		origin_collection_id: T::CollectionId,
		origin_asset_id: T::ItemId,
		received_collection_id: T::CollectionId,
		received_asset_id: T::ItemId,
	}

	/// Structure of received collections, contains origin parachain id, origin collection id, and
	/// received collection id.
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Default)]
	#[scale_info(skip_type_params(T, I))]
	pub struct ReceivedCols<T: Config<I>, I: 'static = ()> {
		origin_para_id: ParaId,
		origin_collection_id: T::CollectionId,
		received_collection_id: T::CollectionId,
	}

	/// Storage for sent assets, contains origin collection id and origin asset id as tuple key and
	/// SentStruct as value.
	#[pallet::storage]
	#[pallet::getter(fn sent_assets)]
	pub type SentAssets<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, (T::CollectionId, T::ItemId), SentStruct<T, I>>;

	/// Storage for received assets, contains received collection id as tuple key and ReceivedStruct
	/// as value.
	#[pallet::storage]
	#[pallet::getter(fn received_assets)]
	pub type ReceivedAssets<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, (T::CollectionId, T::ItemId), ReceivedStruct<T, I>>;

	/// Storage for sent collections, contains origin collection id as key and SentCols as value.
	#[pallet::storage]
	#[pallet::getter(fn received_collections)]
	pub type ReceivedCollections<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::CollectionId, ReceivedCols<T, I>>;

	/// Storage holding proposal ID, it is incremented each time a new proposal is created.
	#[pallet::storage]
	#[pallet::getter(fn next_proposal_id)]
	pub type NextProposalId<T: Config<I>, I: 'static = ()> = StorageValue<_, u64, ValueQuery>;

	/// Storage for cross-chain proposals, contains proposal id as key and Proposal structure as
	/// value.
	#[pallet::storage]
	#[pallet::getter(fn cross_chain_proposals)]
	pub type CrossChainProposals<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, u64, Proposal<T, I>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// Event emited when an empty collection is transferred cross-chain.
		CollectionTransferred {
			origin_collection_id: T::CollectionId,
			origin_collection_metadata: BoundedVec<u8, T::StringLimit>,
			destination_para_id: ParaId,
		},

		/// Event emited when a collection and its NFTs are transferred cross-chain.
		CollectionAndNFTsTransferred {
			origin_collection_id: T::CollectionId,
			nft_ids: Vec<T::ItemId>,
			destination_para_id: ParaId,
		},

		/// Event emited when a collection and its NFTs with different owners are transferred
		/// cross-chain.
		CollectionAndNFTsDiffTransferred {
			origin_collection_id: T::CollectionId,
			nfts: Vec<(T::ItemId, AccountIdLookupOf<T>, BoundedVec<u8, T::StringLimit>)>,
			destination_para_id: ParaId,
			to_address: AccountIdLookupOf<T>,
		},

		/// Event emited when collection cross-chain transfer fails.
		CollectionFailedToXCM {
			e: SendError,
			collection_id: T::CollectionId,
			owner: T::AccountId,
			destination: ParaId,
		},

		/// Event emited when collection metadata update prompt is transferred cross-chain.
		CollectionMetadataSent {
			collection_id: T::CollectionId,
			proposed_data: BoundedVec<u8, T::StringLimit>,
			owner: T::AccountId,
			destination: ParaId,
		},

		/// Event emited when cross-chain collection metadata update prompt transfer fails.
		CollectionMetadataFailedToXCM {
			e: SendError,
			collection_id: T::CollectionId,
			proposed_data: BoundedVec<u8, T::StringLimit>,
			owner: T::AccountId,
			destination: ParaId,
		},

		/// Event emited when cross-chain collection burn prompt transfer fails.
		CollectionBurnFailedToXCM {
			e: SendError,
			collection_id: T::CollectionId,
			burn_data: pallet_nfts::DestroyWitness,
			owner: T::AccountId,
			destination: ParaId,
		},

		/// Event emited when collection burn prompt is transferred cross-chain.
		CollectionBurnSent {
			collection_id: T::CollectionId,
			burn_data: pallet_nfts::DestroyWitness,
			owner: T::AccountId,
			destination: ParaId,
		},

		/// Event emited when cross-chain collection ownership change prompt transfer fails.
		CollectionOwnershipFailedToXCM {
			e: SendError,
			collection_id: T::CollectionId,
			proposed_owner: AccountIdLookupOf<T>,
			destination: ParaId,
		},

		/// Event emited when collection ownership change prompt is transferred cross-chain.
		CollectionOwnershipSent {
			collection_id: T::CollectionId,
			proposed_owner: AccountIdLookupOf<T>,
			destination: ParaId,
		},

		/// Event emited on destination chain, when empty collection is received.
		CollectionReceived {
			origin_collection_id: T::CollectionId,
			received_collection_id: T::CollectionId,
			to_address: AccountIdLookupOf<T>,
		},

		/// Event emited on destination chain, when collection with NFTs is already in received
		/// collections storage.
		CollectionAlreadyReceived {
			origin_collection_id: T::CollectionId,
			to_address: AccountIdLookupOf<T>,
		},

		/// Event emited on destination chain, when empty collection fails to be created.
		CollectionCreationFailed { error: DispatchError, owner: AccountIdLookupOf<T> },

		/// Event emited on destination chain, when collection burn prompt fails to execute.
		CollectionBurnFailed {
			error: DispatchErrorWithPostInfo<PostDispatchInfo>,
			collection_id: T::CollectionId,
			owner: AccountIdLookupOf<T>,
		},

		/// Event emited on destination chain, when collection metadata update prompt fails to
		/// execute.
		CollectionMetadataSetFailed {
			error: DispatchError,
			collection_id: T::CollectionId,
			owner: AccountIdLookupOf<T>,
		},

		/// Event emited on destination chain, when collection ownership change prompt fails to
		/// execute.
		CollectionOwnershipTransferFailed {
			error: DispatchError,
			collection_id: T::CollectionId,
			owner: AccountIdLookupOf<T>,
		},

		/// Event emited on destination chain, when collection and its NFT are successfuly
		/// received.
		CollectionWithNftsReceived {
			collection_id: T::CollectionId,
			items: Vec<(T::ItemId, BoundedVec<u8, T::StringLimit>)>,
		},

		/// Event emited on destination chain, when collection and its NFTs with different owners
		/// are successfuly received.
		CollectionWithNftsDiffOwnersReceived {
			collection_id: T::CollectionId,
			items: Vec<(T::ItemId, AccountIdLookupOf<T>, BoundedVec<u8, T::StringLimit>)>,
		},

		/// Event emitted when collection cross-chain transfer proposal is created (Collection
		/// contains NFTs with different owners).
		CollectionTransferProposalCreated {
			proposal_id: u64,
			collection_id: T::CollectionId,
			proposer: T::AccountId,
			destination: ParaId,
		},

		/// Event emitted when a proposal vote is registered
		CrossChainPropoposalVoteRegistered { proposal_id: u64, voter: T::AccountId, vote: Vote },

		/// Event emitted when proposal expired
		ProposalExpired { proposal_id: u64 },

		/// Event emitted when proposal did not pass
		ProposalDidNotPass { proposal_id: u64 },

		/// Event emitted when non-fungible asset is transferred cross-chain
		NFTTransferred {
			origin_collection_id: T::CollectionId,
			origin_asset_id: T::ItemId,
			destination_para_id: ParaId,
			destination_collection_id: T::CollectionId,
			destination_asset_id: T::ItemId,
		},

		/// Event emitted when non-fungible asset is claimed (Its origin collection was sent
		/// cross-chain to same chain).
		NFTClaimed {
			collection_claimed_from: T::CollectionId,
			asset_removed: T::ItemId,
			collection_claimed_to: T::CollectionId,
			asset_claimed: T::ItemId,
		},

		/// Event emitted when cross-chain NFT metadata update prompt transfer fails.
		NFTMetadataFailedToXCM {
			e: SendError,
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			proposed_data: BoundedVec<u8, T::StringLimit>,
			owner: T::AccountId,
			destination: ParaId,
		},

		/// Event emitted when NFT metadata update prompt is transferred cross-chain.
		NFTMetadataSent {
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			proposed_data: BoundedVec<u8, T::StringLimit>,
			owner: T::AccountId,
			destination: ParaId,
		},

		/// Event emitted when cross-chain NFT burn prompt transfer fails.
		NFTBurnFailedToXCM {
			e: SendError,
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			owner: T::AccountId,
			destination: ParaId,
		},

		/// Event emitted when NFT burn prompt is transferred cross-chain.
		NFTBurnSent {
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			owner: T::AccountId,
			destination: ParaId,
		},

		/// Event emitted when cross-chain NFT ownership change prompt transfer fails.
		NFTOwnershipFailedToXCM {
			e: SendError,
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			proposed_owner: AccountIdLookupOf<T>,
			destination: ParaId,
		},

		/// Event emitted when NFT ownership change prompt is transferred cross-chain.
		NFTOwnershipSent {
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			proposed_owner: AccountIdLookupOf<T>,
			destination: ParaId,
		},

		/// Event emitted on destination chain, when NFT burn prompt fails to execute.
		NFTBurnFailed {
			error: DispatchError,
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			owner: AccountIdLookupOf<T>,
		},

		/// Event emitted on destination chain, when NFT metadata update prompt fails to execute.
		NFTMetadataSetFailed {
			error: DispatchError,
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			owner: AccountIdLookupOf<T>,
		},

		/// Event emitted on destination chain, when NFT ownership change prompt fails to execute.
		NFTOwnershipTransferFailed {
			error: DispatchError,
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			owner: AccountIdLookupOf<T>,
		},

		/// Event emitted on destination chain, when NFT fails to be minted.
		NFTMintFailed {
			error: DispatchError,
			collection_id: T::CollectionId,
			asset_id: T::ItemId,
			owner: AccountIdLookupOf<T>,
		},

		/// Event emitted on destination chain, when NFT is successfully received along with
		/// metadata if provided.
		NFTReceived {
			origin_collection_id: T::CollectionId,
			origin_asset_id: T::ItemId,
			received_collection_id: T::CollectionId,
			received_asset_id: T::ItemId,
			to_address: AccountIdLookupOf<T>,
		},

		/// Event emitted on destination chain, when received NFT is NFT that was previously sent
		/// cross-chain.
		NFTReturnedToOrigin {
			returned_from_collection_id: T::CollectionId,
			returned_from_asset_id: T::ItemId,
			to_address: T::AccountId,
		},

		/// Event emitted when collection fails to mint on destination chain
		CollectionMintFailed { error: DispatchError },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Error returned when collection does not exist.
		CollectionDoesNotExist,

		/// Error returned when NFT does not exist.
		NFTDoesNotExist,

		/// Error returned NFT already exist.
		NFTExists,

		/// Error returned when cross-chain proposal already exists.
		ProposalAlreadyExists,

		/// Error returned when account is not collection owner.
		NotCollectionOwner,

		/// Error returned when same NFT is already received.
		NFTAlreadyReceived,

		/// Error returned when proposal is expired and couldn't be voted on anymore.
		ProposalExpired,

		/// Error returned when proposal is still active, so cross-chain transfer cannot be
		/// initiated.
		ProposalStillActive,

		/// Error returned when proposal does not exist.
		ProposalDoesNotExist,

		/// Error returned when proposal did not pass.
		ProposalDidNotPass,

		/// Error returned when user has already voted the same vote.
		AlreadyVotedThis,

		/// Error returned when maximum number of owners is reached.
		MaxOwnersReached,

		/// Error returned when user is not NFT owner.
		NotNFTOwner,

		/// Error returned when NFT is not received, but user wants to claim it into different
		/// collection.
		NFTNotReceived,

		/// Error, that shouldn't happen.
		NoNextCollectionId,

		/// Error returned when user enters wrong origin collection id.
		WrongOriginCollectionAtOrigin,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Transfer a Collection along with its associated metadata / assets to another parachain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `Collection`;
		///
		/// Arguments:
		/// - `origin_collection`: The collection_id of the collection to be transferred.
		/// - `destination_para`: The destination chain ID to which collection is transferred.
		/// - `config`: The config of transferred collection.
		///
		/// On success emits `CollectionTransferred` or `CollectionAndNFTsTransferred`.
		///
		/// _
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn collection_x_transfer(
			origin: OriginFor<T>,
			origin_collection: T::CollectionId,
			destination_para: ParaId,
			config: CollectionConfigFor<T, I>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// See if collection exists
			ensure!(
				pallet_nfts::Collection::<T, I>::contains_key(&origin_collection),
				Error::<T, I>::CollectionDoesNotExist
			);

			// See if user owns the collection
			ensure!(
				pallet_nfts::Pallet::<T, I>::collection_owner(origin_collection.clone())
					.ok_or(Error::<T, I>::CollectionDoesNotExist)? ==
					who.clone(),
				Error::<T, I>::NotCollectionOwner
			);

			// Retrieve collection item IDs
			let mut items = Vec::new();

			for (item_id, _item_details) in
				pallet_nfts::Item::<T, I>::iter_prefix(&origin_collection)
			{
				items.push(item_id);
			}

			// First check if collection contains any metadata if it does, then save it
			let mut collection_metadata = None;
			if pallet_nfts::CollectionMetadataOf::<T, I>::contains_key(&origin_collection) {
				collection_metadata = Some(
					pallet_nfts::CollectionMetadataOf::<T, I>::get(origin_collection).unwrap().data,
				);
			}

			// If collection metadata is not present, then create empty metadata
			if collection_metadata.is_none() {
				collection_metadata = Some(BoundedVec::new());
			}

			// Convert accountId into accountid32
			let account_vec = who.encode();
			ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
			let mut bytes = [0u8; 32];
			bytes.copy_from_slice(&account_vec);

			// Check if the collection is empty (items array is empty)
			if items.is_empty() {
				// Transfer the empty collection to the destination parachain
				match send_xcm::<T::XcmSender>(
					(Parent, Junction::Parachain(destination_para.into())).into(),
					Xcm(vec![
						UnpaidExecution { weight_limit: Unlimited, check_origin: None },
						AliasOrigin(
							xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
								.into(),
						),
						Transact {
							origin_kind: OriginKind::SovereignAccount,
							require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
							call: <T as Config<I>>::RuntimeCall::from(
								Call::<T, I>::parse_collection_empty {
									origin_collection: origin_collection.clone(),
									collection_metadata: collection_metadata.clone().unwrap(),
									config,
								},
							)
							.encode()
							.into(),
						},
					]),
				) {
					Ok((_hash, _cost)) => {
						// If collection was received, remove from received collections
						if ReceivedCollections::<T, I>::contains_key(&origin_collection) {
							ReceivedCollections::<T, I>::remove(&origin_collection);
						}

						// Get collection from the storage
						let collection =
							pallet_nfts::Collection::<T, I>::get(origin_collection).unwrap();

						// Create destroy witness type
						let destroy_witness = DestroyWitness {
							item_metadatas: collection.clone().item_metadatas,
							item_configs: collection.clone().item_configs,
							attributes: collection.attributes,
						};

						// Burn the collection on origin chain
						let _ = pallet_nfts::Pallet::<T, I>::destroy(
							origin.clone(),
							origin_collection.clone(),
							destroy_witness,
						);

						// Emit an success event
						Self::deposit_event(Event::CollectionTransferred {
							origin_collection_id: origin_collection,
							origin_collection_metadata: collection_metadata.unwrap(),
							destination_para_id: destination_para,
						});
					},
					Err(e) => Self::deposit_event(Event::CollectionFailedToXCM {
						e,
						collection_id: origin_collection,
						owner: who.clone(),
						destination: destination_para,
					}),
				}
			} else {
				// Check if all the NFTs are owned by the same owner
				let collection_owner = who.clone();

				for item_id in items.clone() {
					if let Some(nft_owner) =
						pallet_nfts::Pallet::<T, I>::owner(origin_collection, item_id)
					{
						if nft_owner != collection_owner {
							for (_data, proposal) in CrossChainProposals::<T, I>::iter() {
								if proposal.collection_id == origin_collection {
									return Err(Error::<T, I>::ProposalAlreadyExists.into());
								}
							}

							// Find out how many different NFT owners does the collection have
							let mut different_owners = BoundedVec::new();

							for item_id in items.clone() {
								if let Some(nft_owner) =
									pallet_nfts::Pallet::<T, I>::owner(origin_collection, item_id)
								{
									if nft_owner != collection_owner {
										// Check if owner is not present in different owners
										if !different_owners.contains(&nft_owner) {
											different_owners.try_push(nft_owner).ok();
										}
									}
								}
							}

							// Also add collection owner
							different_owners.try_push(collection_owner.clone()).ok();

							let proposal_id = NextProposalId::<T, I>::get();

							if proposal_id == 0 {
								NextProposalId::<T, I>::put(1);
							} else if proposal_id == u64::MAX {
								NextProposalId::<T, I>::put(1);
							} else {
								NextProposalId::<T, I>::put(proposal_id + 1);
							}

							let block_n: BlockNumberFor<T> =
								frame_system::Pallet::<T>::block_number();

							let proposal = Proposal::<T, I> {
								proposal_id,
								collection_id: origin_collection,
								proposed_collection_owner: who.clone(),
								proposed_destination_config: config,
								proposed_destination_para: destination_para,
								owners: different_owners,
								number_of_votes: Votes {
									aye: BoundedVec::new(),
									nay: BoundedVec::new(),
								},
								end_time: block_n + T::ProposalTimeInBlocks::get().into(),
							};

							<CrossChainProposals<T, I>>::insert(proposal_id, proposal);

							Self::deposit_event(Event::CollectionTransferProposalCreated {
								proposal_id,
								collection_id: origin_collection,
								proposer: who.clone(),
								destination: destination_para,
							});

							return Ok(().into());
						}
					}
				}

				// We get there, because collection owner is the same as NFT owners
				let mut collection_metadata = None;

				if pallet_nfts::CollectionMetadataOf::<T, I>::contains_key(&origin_collection) {
					collection_metadata = Some(
						pallet_nfts::CollectionMetadataOf::<T, I>::get(origin_collection)
							.unwrap()
							.data,
					);
				}

				if collection_metadata.is_none() {
					collection_metadata = Some(BoundedVec::new());
				}

				// Get NFT configs
				let mut nft_metadata = Vec::new();
				for item_id in items.clone() {
					if pallet_nfts::ItemMetadataOf::<T, I>::contains_key(
						&origin_collection,
						item_id,
					) {
						let item_details =
							pallet_nfts::ItemMetadataOf::<T, I>::get(origin_collection, item_id)
								.unwrap()
								.data;
						nft_metadata.push((item_id, item_details));
					} else {
						// Add empty metadata
						nft_metadata.push((item_id, BoundedVec::new()));
					}
				}

				// Send the collection and nfts along with associated metadata to the destination
				// parachain
				match send_xcm::<T::XcmSender>(
					(Parent, Junction::Parachain(destination_para.into())).into(),
					Xcm(vec![
						UnpaidExecution { weight_limit: Unlimited, check_origin: None },
						AliasOrigin(
							xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
								.into(),
						),
						Transact {
							origin_kind: OriginKind::SovereignAccount,
							require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
							call: <T as Config<I>>::RuntimeCall::from(
								Call::<T, I>::parse_collection_same_owner {
									config,
									origin_collection_id: origin_collection.clone(),
									origin_para: parachain_info::Pallet::<T>::parachain_id(),
									collection_metadata: collection_metadata.unwrap(),
									nfts: nft_metadata,
								},
							)
							.encode()
							.into(),
						},
					]),
				) {
					Ok((_hash, _cost)) => {
						// If collection was received, remove from received collections
						if ReceivedCollections::<T, I>::contains_key(&origin_collection) {
							ReceivedCollections::<T, I>::remove(&origin_collection);
						}

						// Burning the NFTs
						for item_id in items.clone() {
							let _ = pallet_nfts::Pallet::<T, I>::burn(
								origin.clone(),
								origin_collection.clone(),
								item_id,
							);
						}

						// Get collection from the storage
						let collection =
							pallet_nfts::Collection::<T, I>::get(origin_collection).unwrap();

						// Create destroy witness type
						let destroy_witness = DestroyWitness {
							item_metadatas: collection.clone().item_metadatas,
							item_configs: collection.clone().item_configs,
							attributes: collection.attributes,
						};

						// Burning the collection
						let _ = pallet_nfts::Pallet::<T, I>::destroy(
							origin.clone(),
							origin_collection.clone(),
							destroy_witness,
						);

						// Emit a success event
						Self::deposit_event(Event::CollectionAndNFTsTransferred {
							origin_collection_id: origin_collection,
							nft_ids: items,
							destination_para_id: destination_para,
						});
					},
					Err(e) => Self::deposit_event(Event::CollectionFailedToXCM {
						e,
						collection_id: origin_collection,
						owner: who.clone(),
						destination: destination_para,
					}),
				}
			}
			Ok(().into())
		}

		/// Cast a vote on collection cross-chain transfer.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `Asset` or `Collection`;
		///
		/// Arguments:
		/// - `proposal_id`: The cross-chain proposal ID.
		/// - `actual_vote`: Enum type - either Aye or Nay.
		///
		/// On success emits `CrossChainPropoposalVoteRegistered`.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn collection_x_transfer_vote(
			origin: OriginFor<T>,
			proposal_id: u64,
			actual_vote: Vote,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Check if proposal exists
			ensure!(
				CrossChainProposals::<T, I>::contains_key(proposal_id),
				Error::<T, I>::ProposalDoesNotExist
			);

			// Get the proposal
			let mut unwrapped_proposal = CrossChainProposals::<T, I>::get(proposal_id).unwrap();

			// See if the user can vote, check if they are in the owners list
			ensure!(unwrapped_proposal.owners.contains(&who), Error::<T, I>::NotNFTOwner);

			// Check if the proposal is still active
			let block_n: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();

			if block_n > unwrapped_proposal.end_time {
				let number_of_votes = &unwrapped_proposal.number_of_votes.aye.len() +
					&unwrapped_proposal.number_of_votes.nay.len();

				// If proposal did not pass (Less than 50% of votes are aye) remove proposal from
				// storage and emit event.
				if unwrapped_proposal.number_of_votes.aye.len() < number_of_votes / 2 ||
					unwrapped_proposal.number_of_votes.aye.len() == 0 &&
						unwrapped_proposal.number_of_votes.nay.len() == 0
				{
					CrossChainProposals::<T, I>::remove(proposal_id);

					Self::deposit_event(Event::ProposalDidNotPass { proposal_id });

					return Ok(().into());
				}

				CrossChainProposals::<T, I>::remove(proposal_id);

				Self::deposit_event(Event::ProposalExpired { proposal_id });

				return Ok(().into());
			}

			// Check if the user has already voted, if they did, see if they voted the same or
			// different. If same, return error and if different, update the vote
			if unwrapped_proposal.number_of_votes.aye.contains(&who) {
				if actual_vote == Vote::Nay {
					unwrapped_proposal.number_of_votes.aye.retain(|x| x != &who);
					unwrapped_proposal
						.number_of_votes
						.nay
						.try_push(who.clone())
						.map_err(|_| Error::<T, I>::MaxOwnersReached)?;
				} else {
					return Err(Error::<T, I>::AlreadyVotedThis.into());
				}
			} else if unwrapped_proposal.number_of_votes.nay.contains(&who) {
				if actual_vote == Vote::Aye {
					unwrapped_proposal.number_of_votes.nay.retain(|x| x != &who);
					unwrapped_proposal
						.number_of_votes
						.aye
						.try_push(who.clone())
						.map_err(|_| Error::<T, I>::MaxOwnersReached)?;
				} else {
					return Err(Error::<T, I>::AlreadyVotedThis.into());
				}
			} else {
				if actual_vote == Vote::Aye {
					unwrapped_proposal
						.number_of_votes
						.aye
						.try_push(who.clone())
						.map_err(|_| Error::<T, I>::MaxOwnersReached)?;
				} else {
					unwrapped_proposal
						.number_of_votes
						.nay
						.try_push(who.clone())
						.map_err(|_| Error::<T, I>::MaxOwnersReached)?;
				}
			}

			// Update the proposal
			CrossChainProposals::<T, I>::insert(proposal_id, unwrapped_proposal);

			//Emit a success event
			Self::deposit_event(Event::CrossChainPropoposalVoteRegistered {
				proposal_id,
				voter: who.clone(),
				vote: actual_vote,
			});

			Ok(().into())
		}

		/// Transfer a Collection along with its associated metadata & assets owned by different
		/// owners to another parachain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `Collection`;
		///
		/// Prereqiuisites:
		/// - Collection must be associated with proposal that has passed.
		///
		/// Arguments:
		/// - `proposal_id`: The cross-chain proposal ID.
		///
		/// On success emits `CollectionAndNFTsDiffTransferred`.
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn collection_x_transfer_initiate(
			origin: OriginFor<T>,
			proposal_id: u64,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// Check if proposal exists
			ensure!(
				CrossChainProposals::<T, I>::contains_key(proposal_id),
				Error::<T, I>::ProposalDoesNotExist
			);

			//Check if owner of the collection is the one who initiated the transfer
			let proposal = CrossChainProposals::<T, I>::get(proposal_id).unwrap();

			ensure!(
				pallet_nfts::Pallet::<T, I>::collection_owner(proposal.collection_id.clone())
					.ok_or(Error::<T, I>::CollectionDoesNotExist)? ==
					who.clone(),
				Error::<T, I>::NotCollectionOwner
			);

			// Check if the proposal is active or not
			let block_n: BlockNumberFor<T> = frame_system::Pallet::<T>::block_number();

			if block_n < proposal.end_time {
				return Err(Error::<T, I>::ProposalStillActive.into());
			}

			// Check if the proposal passed
			let number_of_votes =
				proposal.number_of_votes.aye.len() + proposal.number_of_votes.nay.len();

			if proposal.number_of_votes.aye.len() < number_of_votes / 2 ||
				proposal.number_of_votes.aye.len() == 0 &&
					proposal.number_of_votes.nay.len() == 0
			{
				Self::deposit_event(Event::ProposalDidNotPass { proposal_id });

				// Remove the proposal
				CrossChainProposals::<T, I>::remove(proposal_id);

				return Ok(().into());
			} else if proposal.number_of_votes.aye.len() >= number_of_votes / 2 {
				// Get the collection metadata
				let mut collection_metadata = Some(BoundedVec::new());

				if pallet_nfts::CollectionMetadataOf::<T, I>::contains_key(
					proposal.collection_id.clone(),
				) {
					collection_metadata = Some(
						pallet_nfts::CollectionMetadataOf::<T, I>::get(
							proposal.collection_id.clone(),
						)
						.unwrap()
						.data,
					);
				}

				// Get NFT metadata
				let mut nft_metadata = Vec::new();
				let mut items = Vec::new();

				for (item_id, _item_details) in
					pallet_nfts::Item::<T, I>::iter_prefix(proposal.collection_id.clone())
				{
					items.push(item_id);
				}

				if items.is_empty() {
					// Remove the proposal
					CrossChainProposals::<T, I>::remove(proposal_id);

					// Transfer through regular transfer function again, because there are no NFTs
					// in the collection
					Self::collection_x_transfer(
						origin.clone(),
						proposal.collection_id,
						proposal.proposed_destination_para,
						proposal.proposed_destination_config.clone(),
					)?;
				}

				for item_id in items.clone() {
					let nft_owner =
						pallet_nfts::Pallet::<T, I>::owner(proposal.collection_id.clone(), item_id)
							.unwrap();
					let unlooked_recipient = T::Lookup::unlookup(nft_owner.clone());

					if pallet_nfts::ItemMetadataOf::<T, I>::contains_key(
						proposal.collection_id.clone(),
						item_id,
					) {
						let item_details = pallet_nfts::ItemMetadataOf::<T, I>::get(
							proposal.collection_id.clone(),
							item_id,
						)
						.unwrap()
						.data;
						nft_metadata.push((item_id, unlooked_recipient.clone(), item_details));
					} else {
						// Add empty metadata
						nft_metadata.push((item_id, unlooked_recipient.clone(), BoundedVec::new()));
					}
				}

				let destination = proposal.proposed_destination_para.clone();
				let unlooked_col_recipient = T::Lookup::unlookup(who.clone());
				let config = proposal.proposed_destination_config.clone();

				// Convert accountId into accountid32
				let account_vec = who.encode();
				ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
				let mut bytes = [0u8; 32];
				bytes.copy_from_slice(&account_vec);

				// Send collection and NFTs along with their metadata to destination parachain
				match send_xcm::<T::XcmSender>(
					(Parent, Junction::Parachain(destination.into())).into(),
					Xcm(vec![
						UnpaidExecution { weight_limit: Unlimited, check_origin: None },
						AliasOrigin(
							xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
								.into(),
						),
						Transact {
							origin_kind: OriginKind::SovereignAccount,
							require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
							call: <T as Config<I>>::RuntimeCall::from(
								Call::<T, I>::parse_collection_diff_owners {
									config,
									origin_collection_id: proposal.collection_id.clone(),
									origin_para: parachain_info::Pallet::<T>::parachain_id(),
									collection_metadata: collection_metadata.unwrap(),
									nfts: nft_metadata.clone(),
								},
							)
							.encode()
							.into(),
						},
					]),
				) {
					Ok((_hash, _cost)) => {
						// If collection was received, remove from received collections
						if ReceivedCollections::<T, I>::contains_key(proposal.collection_id.clone())
						{
							ReceivedCollections::<T, I>::remove(proposal.collection_id.clone());
						}

						// Burning the NFTs
						for item_id in items.clone() {
							//Get NFT owner
							let nft_owner = pallet_nfts::Pallet::<T, I>::owner(
								proposal.collection_id.clone(),
								item_id,
							)
							.unwrap();
							let signed_nft_owner: OriginFor<T> =
								frame_system::RawOrigin::Signed(nft_owner.clone()).into();

							//Burn the NFT
							let _ = pallet_nfts::Pallet::<T, I>::burn(
								signed_nft_owner,
								proposal.collection_id.clone(),
								item_id,
							);
						}

						//Burning the collection
						let collection =
							pallet_nfts::Collection::<T, I>::get(proposal.collection_id.clone())
								.unwrap();

						let destroy_witness = DestroyWitness {
							item_metadatas: collection.clone().item_metadatas,
							item_configs: collection.clone().item_configs,
							attributes: collection.attributes,
						};

						let _ = pallet_nfts::Pallet::<T, I>::destroy(
							origin.clone(),
							proposal.collection_id.clone(),
							destroy_witness,
						);

						// Remove proposal from proposals
						CrossChainProposals::<T, I>::remove(proposal_id);

						// Emit a success event.
						Self::deposit_event(Event::CollectionAndNFTsDiffTransferred {
							origin_collection_id: proposal.collection_id.clone(),
							nfts: nft_metadata.clone(),
							destination_para_id: proposal.proposed_destination_para.clone(),
							to_address: unlooked_col_recipient.clone(),
						});
					},
					Err(e) => Self::deposit_event(Event::CollectionFailedToXCM {
						e,
						collection_id: proposal.collection_id.clone(),
						owner: who.clone(),
						destination: proposal.proposed_destination_para.clone(),
					}),
				}
			}

			Ok(().into())
		}

		/// Transfer an asset along with associated metadata to another parachain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `Asset`;
		///
		/// Arguments:
		/// - `origin_collection`: The collection_id of the collection to be transferred.
		/// - `origin_asset`: The asset_id of the asset to be transferred.
		/// - `destination_para`: The destination chain ID to which collection is transferred.
		/// - `destination_collection`: The collection_id of the collection that the asset have to
		///   be received into.
		/// - `destination_asset`: The asset_id of the asset to be received.
		///
		/// On success emits `NFTTransferred`.
		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nft_x_transfer(
			origin: OriginFor<T>,
			origin_collection: T::CollectionId,
			origin_asset: T::ItemId,
			destination_para: ParaId,
			destination_collection: T::CollectionId,
			destination_asset: T::ItemId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// See if collection exists
			ensure!(
				pallet_nfts::Collection::<T, I>::contains_key(&origin_collection),
				Error::<T, I>::CollectionDoesNotExist
			);

			// See if item exists
			ensure!(
				pallet_nfts::Item::<T, I>::contains_key(&origin_collection, &origin_asset),
				Error::<T, I>::NFTDoesNotExist
			);

			// See if user owns the item
			ensure!(
				pallet_nfts::Pallet::<T, I>::owner(origin_collection.clone(), origin_asset.clone())
					.ok_or(Error::<T, I>::NFTDoesNotExist)? ==
					who.clone(),
				Error::<T, I>::NotNFTOwner
			);

			// Get Item data
			let mut metadata = Some(BoundedVec::new());

			if pallet_nfts::ItemMetadataOf::<T, I>::contains_key(&origin_collection, &origin_asset)
			{
				metadata = pallet_nfts::ItemMetadataOf::<T, I>::get(
					origin_collection.clone(),
					origin_asset.clone(),
				)
				.map(|i| i.data);
			}

			// Convert accountId into accountid32
			let account_vec = who.encode();
			ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
			let mut bytes = [0u8; 32];
			bytes.copy_from_slice(&account_vec);

			// Send the asset along with associated metadata cross-chain
			match send_xcm::<T::XcmSender>(
				(Parent, Junction::Parachain(destination_para.into())).into(),
				Xcm(vec![
					UnpaidExecution { weight_limit: Unlimited, check_origin: None },
					AliasOrigin(
						xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
							.into(),
					),
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
						call: <T as Config<I>>::RuntimeCall::from(
							Call::<T, I>::parse_nft_transfer {
								origin_collection: origin_collection.clone(),
								origin_item: origin_asset.clone(),
								collection: destination_collection.clone(),
								item: destination_asset.clone(),
								data: metadata.unwrap(),
								origin_chain: parachain_info::Pallet::<T>::parachain_id(),
							},
						)
						.encode()
						.into(),
					},
				]),
			) {
				Ok((_hash, _cost)) => {
					// If in received list, burn asset and remove from received list
					if ReceivedAssets::<T, I>::contains_key(&(
						origin_collection.clone(),
						origin_asset.clone(),
					)) {
						let received = ReceivedAssets::<T, I>::get(&(
							origin_collection.clone(),
							origin_asset.clone(),
						))
						.unwrap();

						SentAssets::<T, I>::insert(
							(origin_collection.clone(), origin_asset.clone()),
							SentStruct {
								origin_para_id: received.origin_para_id,
								origin_collection_id: received.origin_collection_id,
								origin_asset_id: received.origin_asset_id,
								destination_collection_id: destination_collection.clone(),
								destination_asset_id: destination_asset.clone(),
							},
						);

						// Remove from received assets
						ReceivedAssets::<T, I>::remove(&(
							origin_collection.clone(),
							origin_asset.clone(),
						));

						// Burn the asset
						let _ = pallet_nfts::Pallet::<T, I>::burn(
							origin.clone(),
							origin_collection.clone(),
							origin_asset.clone(),
						);
					}
					//Only remove asset metadata, because we are sending from origin chain
					else {
						let col_owner = pallet_nfts::Pallet::<T, I>::collection_owner(
							origin_collection.clone(),
						)
						.unwrap();
						let signed_col: OriginFor<T> =
							frame_system::RawOrigin::Signed(col_owner.clone()).into();

						let _ = pallet_nfts::Pallet::<T, I>::clear_metadata(
							signed_col.clone(),
							origin_collection.clone(),
							origin_asset.clone(),
						);

						SentAssets::<T, I>::insert(
							(origin_collection.clone(), origin_asset.clone()),
							SentStruct {
								origin_para_id: parachain_info::Pallet::<T>::parachain_id(),
								origin_collection_id: origin_collection.clone(),
								origin_asset_id: origin_asset.clone(),
								destination_collection_id: destination_collection.clone(),
								destination_asset_id: destination_asset.clone(),
							},
						);
					}
					//Emit a success event
					Self::deposit_event(Event::NFTTransferred {
						origin_collection_id: origin_collection.clone(),
						origin_asset_id: origin_asset.clone(),
						destination_para_id: destination_para,
						destination_collection_id: destination_collection.clone(),
						destination_asset_id: destination_asset.clone(),
					});
				},
				Err(e) => Self::deposit_event(Event::CollectionFailedToXCM {
					e,
					collection_id: origin_collection.clone(),
					owner: who.clone(),
					destination: destination_para.clone(),
				}),
			}
			Ok(().into())
		}

		/// Claim cross-chain sent asset if its origin collection was also sent to same destination
		/// chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the asset in the `Current collection` and owner of the asset in the
		///   `Origin collection`;
		///
		/// Arguments:
		/// - `origin_collection_at_destination`: The collection_id at the destination of the
		///   collection that was transferred (Id it has at destination chain).
		/// - `origin_collection_at_origin`: The origin collection_id of the collection that was
		///   transferred (Id it had at origin chain).
		/// - `origin_asset_at_destination`: The origin asset id at the origin collection that is
		///   delivered.
		/// - `current_collection`: The collection_id of the collection that the asset have been
		///   delivered into.
		/// - `current_asset`: The current asset_id of the asset in current collection.
		///
		/// On success emits `NFTClaimed`.
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nft_x_claim(
			origin: OriginFor<T>,
			origin_collection_at_destination: T::CollectionId,
			origin_collection_at_origin: T::CollectionId,
			origin_asset_at_destination: T::ItemId,
			current_collection: T::CollectionId,
			current_asset: T::ItemId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			// See if collection exists
			ensure!(
				pallet_nfts::Collection::<T, I>::contains_key(&current_collection),
				Error::<T, I>::CollectionDoesNotExist
			);

			// See if origin collection exists
			ensure!(
				pallet_nfts::Collection::<T, I>::contains_key(&origin_collection_at_destination),
				Error::<T, I>::CollectionDoesNotExist
			);

			// See if origin collection at origin is the same as in received collection
			ensure!(
				ReceivedCollections::<T, I>::get(&origin_collection_at_destination)
					.unwrap()
					.origin_collection_id ==
					origin_collection_at_origin,
				Error::<T, I>::WrongOriginCollectionAtOrigin
			);

			// See if current asset is in received assets
			ensure!(
				ReceivedAssets::<T, I>::contains_key(&(
					current_collection.clone(),
					current_asset.clone()
				)),
				Error::<T, I>::NFTNotReceived
			);

			// See if item in origin collection exists
			ensure!(
				pallet_nfts::Item::<T, I>::contains_key(
					&origin_collection_at_destination,
					&origin_asset_at_destination
				),
				Error::<T, I>::NFTDoesNotExist
			);

			// See if user owns the item
			ensure!(
				pallet_nfts::Pallet::<T, I>::owner(
					origin_collection_at_destination.clone(),
					origin_asset_at_destination.clone()
				)
				.ok_or(Error::<T, I>::NFTDoesNotExist)? ==
					who.clone(),
				Error::<T, I>::NotNFTOwner
			);

			// See if user owns the current asset
			ensure!(
				pallet_nfts::Pallet::<T, I>::owner(
					current_collection.clone(),
					current_asset.clone()
				)
				.ok_or(Error::<T, I>::NFTDoesNotExist)? ==
					who.clone(),
				Error::<T, I>::NotNFTOwner
			);

			// Claim the asset
			let mut metadata = Some(BoundedVec::new());

			if pallet_nfts::ItemMetadataOf::<T, I>::contains_key(
				current_collection.clone(),
				current_asset.clone(),
			) {
				metadata = pallet_nfts::ItemMetadataOf::<T, I>::get(
					current_collection.clone(),
					current_asset.clone(),
				)
				.map(|i| i.data);
			}

			// Burn the current asset
			let _ = pallet_nfts::Pallet::<T, I>::burn(
				origin.clone(),
				current_collection.clone(),
				current_asset.clone(),
			);

			// Add the metadata to the old asset location
			if metadata.is_some() {
				let _ = pallet_nfts::Pallet::<T, I>::set_metadata(
					origin.clone(),
					origin_collection_at_destination.clone(),
					origin_asset_at_destination.clone(),
					metadata.unwrap(),
				);
			}

			// Remove asset from received
			ReceivedAssets::<T, I>::remove(&(current_collection.clone(), current_asset.clone()));

			// Emit a success event
			Self::deposit_event(Event::NFTClaimed {
				collection_claimed_from: current_collection.clone(),
				asset_removed: current_asset.clone(),
				collection_claimed_to: origin_collection_at_destination.clone(),
				asset_claimed: origin_asset_at_destination.clone(),
			});

			Ok(().into())
		}

		/// Update collection metadata cross-chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `destination collection`;
		///
		/// Arguments:
		/// - `destination_collection_id`: The collection_id at the destination.
		/// - `destination_para`: The recipient parachain ID.
		/// - `data`: The metadata to be added to destination collection.
		///
		/// On success emits `CollectionMetadataSent`.
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn collection_x_update(
			origin: OriginFor<T>,
			destination_collection_id: T::CollectionId,
			destination_para: ParaId,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Convert accountId into accountid32
			let account_vec = who.encode();
			ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
			let mut bytes = [0u8; 32];
			bytes.copy_from_slice(&account_vec);

			// Send the prompt to update collection metadata
			match send_xcm::<T::XcmSender>(
				(Parent, Junction::Parachain(destination_para.into())).into(),
				Xcm(vec![
					UnpaidExecution { weight_limit: Unlimited, check_origin: None },
					AliasOrigin(
						xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
							.into(),
					),
					Transact {
						origin_kind: OriginKind::Native,
						require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
						call: <T as Config<I>>::RuntimeCall::from(
							Call::<T, I>::parse_collection_metadata {
								collection: destination_collection_id.clone(),
								data: data.clone(),
							},
						)
						.encode()
						.into(),
					},
				]),
			) {
				Ok((_hash, _cost)) => {
					// Emit event about sucessful metadata send
					Self::deposit_event(Event::CollectionMetadataSent {
						collection_id: destination_collection_id.clone(),
						proposed_data: data.clone(),
						owner: who.clone(),
						destination: destination_para.clone(),
					});
				},
				Err(e) => Self::deposit_event(Event::CollectionMetadataFailedToXCM {
					e,
					collection_id: destination_collection_id.clone(),
					proposed_data: data.clone(),
					owner: who.clone(),
					destination: destination_para.clone(),
				}),
			}
			Ok(().into())
		}

		/// Update NFT metadata cross-chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `destination collection`;
		///
		/// Arguments:
		/// - `destination_collection_id`: The collection_id at the destination.
		/// - `destination_asset_id`: The asset_id at the destination.
		/// - `destination_para`: The recipient parachain ID.
		/// - `data`: The metadata to be added to destination collection.
		///
		/// On success emits `NFTMetadataSent`.
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nft_x_update(
			origin: OriginFor<T>,
			destination_collection_id: T::CollectionId,
			destination_asset_id: T::ItemId,
			destination_para: ParaId,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Convert accountId into accountid32
			let account_vec = who.encode();
			ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
			let mut bytes = [0u8; 32];
			bytes.copy_from_slice(&account_vec);

			// Send the prompt to update NFT metadata
			match send_xcm::<T::XcmSender>(
				(Parent, Junction::Parachain(destination_para.into())).into(),
				Xcm(vec![
					UnpaidExecution { weight_limit: Unlimited, check_origin: None },
					AliasOrigin(
						xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
							.into(),
					),
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
						call: <T as Config<I>>::RuntimeCall::from(
							Call::<T, I>::parse_nft_metadata {
								collection: destination_collection_id.clone(),
								item: destination_asset_id.clone(),
								data: data.clone(),
							},
						)
						.encode()
						.into(),
					},
				]),
			) {
				Ok((_hash, _cost)) => {
					// Emit event about sucessful metadata send
					Self::deposit_event(Event::NFTMetadataSent {
						collection_id: destination_collection_id.clone(),
						asset_id: destination_asset_id.clone(),
						proposed_data: data.clone(),
						owner: who.clone(),
						destination: destination_para.clone(),
					});
				},
				Err(e) => Self::deposit_event(Event::NFTMetadataFailedToXCM {
					e,
					collection_id: destination_collection_id.clone(),
					asset_id: destination_asset_id.clone(),
					proposed_data: data.clone(),
					owner: who.clone(),
					destination: destination_para.clone(),
				}),
			}
			Ok(().into())
		}

		/// Prompt to burn empty collection cross-chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `destination collection`;
		///
		/// Arguments:
		/// - `destination_collection_id`: The collection_id at the destination.
		/// - `destination_para`: The recipient parachain ID.
		/// - `witness_data`: The amount of NFTs, metadatas and configs in the collection (Needs to
		///   be all zeros for successful burn).
		///
		/// On success emits `CollectionBurnSent`.
		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn collection_x_burn(
			origin: OriginFor<T>,
			destination_collection_id: T::CollectionId,
			destination_para: ParaId,
			witnes_data: pallet_nfts::DestroyWitness,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Convert accountId into accountid32
			let account_vec = who.encode();
			ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
			let mut bytes = [0u8; 32];
			bytes.copy_from_slice(&account_vec);

			// Send the prompt to burn collection
			match send_xcm::<T::XcmSender>(
				(Parent, Junction::Parachain(destination_para.into())).into(),
				Xcm(vec![
					UnpaidExecution { weight_limit: Unlimited, check_origin: None },
					AliasOrigin(
						xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
							.into(),
					),
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
						call: <T as Config<I>>::RuntimeCall::from(Call::parse_collection_burn {
							collection_to_burn: destination_collection_id.clone(),
							witness_data: witnes_data.clone(),
						})
						.encode()
						.into(),
					},
				]),
			) {
				Ok((_hash, _cost)) => {
					// Emit event about sucessful burn prompt transfer
					Self::deposit_event(Event::CollectionBurnSent {
						collection_id: destination_collection_id.clone(),
						burn_data: witnes_data.clone(),
						owner: who.clone(),
						destination: destination_para.clone(),
					});
				},
				Err(e) => Self::deposit_event(Event::CollectionBurnFailedToXCM {
					e,
					collection_id: destination_collection_id.clone(),
					burn_data: witnes_data.clone(),
					owner: who.clone(),
					destination: destination_para.clone(),
				}),
			}

			Ok(().into())
		}

		/// Prompt to burn NFT cross-chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `destination NFT`;
		///
		/// Arguments:
		/// - `destination_collection_id`: The collection_id at the destination.
		/// - `destination_asset_id`: The asset_id at the destination.
		/// - `destination_para`: The recipient parachain ID.
		///
		/// On success emits `NFTBurnSent`.
		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nft_x_burn(
			origin: OriginFor<T>,
			destination_collection_id: T::CollectionId,
			destination_asset_id: T::ItemId,
			destination_para: ParaId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Convert accountId into accountid32
			let account_vec = who.encode();
			ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
			let mut bytes = [0u8; 32];
			bytes.copy_from_slice(&account_vec);

			// Send the prompt to burn NFT
			match send_xcm::<T::XcmSender>(
				(Parent, Junction::Parachain(destination_para.into())).into(),
				Xcm(vec![
					UnpaidExecution { weight_limit: Unlimited, check_origin: None },
					AliasOrigin(
						xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
							.into(),
					),
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
						call: <T as Config<I>>::RuntimeCall::from(Call::<T, I>::parse_nft_burn {
							collection: destination_collection_id.clone(),
							item: destination_asset_id.clone(),
						})
						.encode()
						.into(),
					},
				]),
			) {
				Ok((_hash, _cost)) => {
					// Emit event about sucessful metadata send
					Self::deposit_event(Event::NFTBurnSent {
						collection_id: destination_collection_id.clone(),
						asset_id: destination_asset_id.clone(),
						owner: who.clone(),
						destination: destination_para.clone(),
					});
				},
				Err(e) => Self::deposit_event(Event::NFTBurnFailedToXCM {
					e,
					collection_id: destination_collection_id.clone(),
					asset_id: destination_asset_id.clone(),
					owner: who.clone(),
					destination: destination_para.clone(),
				}),
			}

			Ok(().into())
		}

		/// Prompt to change collection owner cross-chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `destination collection`;
		/// - the New owner must agree to the ownership change by executing function
		///   setAcceptOwnership(maybeCollection)
		///
		/// Arguments:
		/// - `destination_collection_id`: The collection_id at the destination.
		/// - `destination_para`: The recipient parachain ID.
		/// - `destination_account`: The destination account that will receive collection.
		///
		/// On success emits `CollectionOwnershipSent`.
		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn collection_x_change_owner(
			origin: OriginFor<T>,
			destination_collection_id: T::CollectionId,
			destination_para: ParaId,
			destination_account: AccountIdLookupOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Convert accountId into accountid32
			let account_vec = who.encode();
			ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
			let mut bytes = [0u8; 32];
			bytes.copy_from_slice(&account_vec);

			// Send the prompt to change collection owner
			match send_xcm::<T::XcmSender>(
				(Parent, Junction::Parachain(destination_para.into())).into(),
				Xcm(vec![
					UnpaidExecution { weight_limit: Unlimited, check_origin: None },
					AliasOrigin(
						xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
							.into(),
					),
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
						call: <T as Config<I>>::RuntimeCall::from(
							Call::<T, I>::parse_collection_owner {
								new_owner: destination_account.clone(),
								collection: destination_collection_id.clone(),
							},
						)
						.encode()
						.into(),
					},
				]),
			) {
				Ok((_hash, _cost)) => {
					// Emit event about sucessful metadata send
					Self::deposit_event(Event::CollectionOwnershipSent {
						collection_id: destination_collection_id.clone(),
						proposed_owner: destination_account.clone(),
						destination: destination_para.clone(),
					});
				},
				Err(e) => Self::deposit_event(Event::CollectionOwnershipFailedToXCM {
					e,
					collection_id: destination_collection_id.clone(),
					proposed_owner: destination_account.clone(),
					destination: destination_para.clone(),
				}),
			}

			Ok(().into())
		}

		/// Prompt to change NFT owner cross-chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `destination asset`;
		///
		/// Arguments:
		/// - `destination_collection_id`: The collection_id at the destination.
		/// - `destination_asset_id`: The asset_id at the destination.
		/// - `destination_para`: The recipient parachain ID.
		/// - `destination_account`: The destination account that will receive collection.
		///
		/// On success emits `NFTOwnershipSent`.
		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nft_x_change_owner(
			origin: OriginFor<T>,
			destination_collection_id: T::CollectionId,
			destination_asset_id: T::ItemId,
			destination_para: ParaId,
			destination_account: AccountIdLookupOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// Convert accountId into accountid32
			let account_vec = who.encode();
			ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
			let mut bytes = [0u8; 32];
			bytes.copy_from_slice(&account_vec);

			// Send the prompt to change NFT owner
			match send_xcm::<T::XcmSender>(
				(Parent, Junction::Parachain(destination_para.into())).into(),
				Xcm(vec![
					UnpaidExecution { weight_limit: Unlimited, check_origin: None },
					AliasOrigin(
						xcm::latest::prelude::AccountId32 { id: bytes.into(), network: None }
							.into(),
					),
					Transact {
						origin_kind: OriginKind::SovereignAccount,
						require_weight_at_most: Weight::from_parts(1_000_000_000, 64 * 1024),
						call: <T as Config<I>>::RuntimeCall::from(Call::<T, I>::parse_nft_owner {
							new_owner: destination_account.clone(),
							collection: destination_collection_id.clone(),
							item: destination_asset_id.clone(),
						})
						.encode()
						.into(),
					},
				]),
			) {
				Ok((_hash, _cost)) => {
					// Emit event about sucessful metadata send
					Self::deposit_event(Event::NFTOwnershipSent {
						collection_id: destination_collection_id.clone(),
						asset_id: destination_asset_id.clone(),
						proposed_owner: destination_account.clone(),
						destination: destination_para.clone(),
					});
				},
				Err(e) => Self::deposit_event(Event::NFTOwnershipFailedToXCM {
					e,
					collection_id: destination_collection_id.clone(),
					asset_id: destination_asset_id.clone(),
					proposed_owner: destination_account.clone(),
					destination: destination_para.clone(),
				}),
			}

			Ok(().into())
		}

		/// Receive function for collection_x_transfer function.
		///
		/// Shouldn't be used as a regular call.
		///
		/// On success emits `CollectionReceived`.
		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_collection_empty(
			origin: OriginFor<T>,
			origin_collection: T::CollectionId,
			collection_metadata: BoundedVec<u8, T::StringLimit>,
			config: CollectionConfigFor<T, I>,
		) -> DispatchResultWithPostInfo {
			let signed_origin = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(signed_origin.clone());

			// Create the collection
			match pallet_nfts::Pallet::<T, I>::create(
				origin.clone(),
				signed_origin_lookup.clone(),
				config.clone(),
			) {
				Ok(_) => {},
				Err(e) => {
					// Deposit event indicating failure to create collection
					Self::deposit_event(Event::CollectionCreationFailed {
						owner: signed_origin_lookup.clone(),
						error: e,
					});
				},
			}

			// Set the collection metadata if not empty
			if !collection_metadata.is_empty() {
				match pallet_nfts::Pallet::<T, I>::set_collection_metadata(
					origin.clone(),
					origin_collection.clone(),
					collection_metadata.clone(),
				) {
					Ok(_) => {},
					Err(e) => {
						// Deposit event indicating failure to set metadata
						Self::deposit_event(Event::CollectionMetadataSetFailed {
							collection_id: origin_collection.clone(),
							owner: signed_origin_lookup.clone(),
							error: e,
						});
					},
				}
			}

			// Emit a success event
			Self::deposit_event(Event::CollectionReceived {
				origin_collection_id: origin_collection.clone(),
				received_collection_id: origin_collection.clone(),
				to_address: signed_origin_lookup,
			});

			Ok(().into())
		}

		/// Receive function for collection_x_burn function.
		///
		/// Doesn't differ from nfts pallet destroy function.
		///
		/// On success emits regular destroy function events.
		#[pallet::call_index(12)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_collection_burn(
			origin: OriginFor<T>,
			collection_to_burn: T::CollectionId,
			witness_data: pallet_nfts::DestroyWitness,
		) -> DispatchResultWithPostInfo {
			let signed_origin = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(signed_origin.clone());

			match pallet_nfts::Pallet::<T, I>::destroy(
				origin.clone(),
				collection_to_burn.clone(),
				witness_data.clone(),
			) {
				Ok(_) => {},
				Err(e) => {
					// Deposit event indicating failure to burn collection
					Self::deposit_event(Event::CollectionBurnFailed {
						owner: signed_origin_lookup.clone(),
						collection_id: collection_to_burn.clone(),
						error: e,
					});
				},
			}

			Ok(().into())
		}

		/// Receive function for collection_x_update function.
		///
		/// Doesn't differ from nfts pallet setCollectionMetadara function.
		///
		/// On success emits regular setCollectionMetadata function events.
		#[pallet::call_index(13)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_collection_metadata(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResultWithPostInfo {
			let signed_origin = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(signed_origin.clone());

			match pallet_nfts::Pallet::<T, I>::set_collection_metadata(
				origin.clone(),
				collection.clone(),
				data.clone(),
			) {
				Ok(_) => {},
				Err(e) => {
					// Deposit event indicating failure to set metadata
					Self::deposit_event(Event::CollectionMetadataSetFailed {
						collection_id: collection.clone(),
						owner: signed_origin_lookup.clone(),
						error: e,
					});
				},
			}

			Ok(().into())
		}

		/// Receive function for collection_x_change_owner function.
		///
		/// Doesn't differ from nfts pallet transferOwnership function.
		///
		/// On success emits regular transferOwnership function events.
		#[pallet::call_index(14)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_collection_owner(
			origin: OriginFor<T>,
			new_owner: AccountIdLookupOf<T>,
			collection: T::CollectionId,
		) -> DispatchResultWithPostInfo {
			let signed_origin = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(signed_origin.clone());

			match pallet_nfts::Pallet::<T, I>::transfer_ownership(
				origin.clone(),
				collection.clone(),
				new_owner.clone(),
			) {
				Ok(_) => {},
				Err(e) => {
					// Deposit event indicating failure to transfer ownership
					Self::deposit_event(Event::CollectionOwnershipTransferFailed {
						collection_id: collection.clone(),
						owner: signed_origin_lookup.clone(),
						error: e,
					});
				},
			}

			Ok(().into())
		}

		/// Receive function for nft_x_burn function.
		///
		/// Doesn't differ from nfts pallet burn function.
		///
		/// On success emits regular burn function events.
		#[pallet::call_index(15)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_nft_burn(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
		) -> DispatchResultWithPostInfo {
			let signed_origin = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(signed_origin.clone());

			match pallet_nfts::Pallet::<T, I>::burn(
				origin.clone(),
				collection.clone(),
				item.clone(),
			) {
				Ok(_) => {},
				Err(e) => {
					// Deposit event indicating failure to burn NFT
					Self::deposit_event(Event::NFTBurnFailed {
						collection_id: collection.clone(),
						asset_id: item.clone(),
						owner: signed_origin_lookup.clone(),
						error: e,
					});
				},
			}

			Ok(().into())
		}

		/// Receive function for  nft_x_update function.
		///
		/// Doesn't differ from nfts pallet setMetadata function.
		///
		/// On success emits regular setMetadata function events.
		#[pallet::call_index(16)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_nft_metadata(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResultWithPostInfo {
			let signed_origin = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(signed_origin.clone());

			match pallet_nfts::Pallet::<T, I>::set_metadata(
				origin.clone(),
				collection.clone(),
				item.clone(),
				data.clone(),
			) {
				Ok(_) => {},
				Err(e) => {
					// Deposit event indicating failure to set metadata
					Self::deposit_event(Event::NFTMetadataSetFailed {
						collection_id: collection.clone(),
						asset_id: item.clone(),
						owner: signed_origin_lookup.clone(),
						error: e,
					});
				},
			}

			Ok(().into())
		}

		/// Receive function for  nft_x_change_owner function.
		///
		/// Doesn't differ from nfts pallet transfer function.
		///
		/// On success emits regular transfer function events.
		#[pallet::call_index(17)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_nft_owner(
			origin: OriginFor<T>,
			new_owner: AccountIdLookupOf<T>,
			collection: T::CollectionId,
			item: T::ItemId,
		) -> DispatchResultWithPostInfo {
			let signed_origin = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(signed_origin.clone());

			match pallet_nfts::Pallet::<T, I>::transfer(
				origin.clone(),
				collection.clone(),
				item.clone(),
				new_owner.clone(),
			) {
				Ok(_) => {},
				Err(e) => {
					// Deposit event indicating failure to transfer ownership
					Self::deposit_event(Event::NFTOwnershipTransferFailed {
						collection_id: collection.clone(),
						asset_id: item.clone(),
						owner: signed_origin_lookup.clone(),
						error: e,
					});
				},
			}

			Ok(().into())
		}

		/// Receive function for  nft_x_transfer function.
		///
		/// Shouldn't be used as a regular call.
		///
		/// On success emits `NFTReceived` or `NFTReturnedToOrigin` events.
		#[pallet::call_index(18)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_nft_transfer(
			origin: OriginFor<T>,
			collection: T::CollectionId,
			item: T::ItemId,
			data: BoundedVec<u8, T::StringLimit>,
			origin_collection: T::CollectionId,
			origin_item: T::ItemId,
			origin_chain: ParaId,
		) -> DispatchResultWithPostInfo {
			let signed_origin = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(signed_origin.clone());

			// Check if the collection exists
			ensure!(
				pallet_nfts::Collection::<T, I>::contains_key(&collection),
				Error::<T, I>::CollectionDoesNotExist
			);

			// Check if not in receiving assets
			ensure!(
				!ReceivedAssets::<T, I>::contains_key(&(collection.clone(), item.clone())),
				Error::<T, I>::NFTAlreadyReceived
			);

			if SentAssets::<T, I>::contains_key(&(collection.clone(), item.clone())) {
				// User returns nft to origin collection, use collection owner to add metadata
				let col_owner =
					pallet_nfts::Pallet::<T, I>::collection_owner(collection.clone()).unwrap();
				let signed_col: OriginFor<T> =
					frame_system::RawOrigin::Signed(col_owner.clone()).into();
				let sent_asset =
					SentAssets::<T, I>::get(&(collection.clone(), item.clone())).unwrap();

				if sent_asset.origin_para_id == parachain_info::Pallet::<T>::parachain_id() {
					// We know we are the origin chain, we can add only metadata
					if !data.is_empty() {
						match pallet_nfts::Pallet::<T, I>::set_metadata(
							signed_col.clone(),
							collection.clone(),
							item.clone(),
							data.clone(),
						) {
							Ok(_) => {},
							Err(e) => {
								// Deposit event indicating failure to set metadata
								Self::deposit_event(Event::NFTMetadataSetFailed {
									collection_id: collection.clone(),
									asset_id: item.clone(),
									owner: signed_origin_lookup.clone(),
									error: e,
								});
							},
						}
					}

					// We also remove sent assets and received assets
					SentAssets::<T, I>::remove(&(collection.clone(), item.clone()));

					// We emit event about return to origin chain
					Self::deposit_event(Event::NFTReturnedToOrigin {
						returned_from_collection_id: collection.clone(),
						returned_from_asset_id: item.clone(),
						to_address: signed_origin.clone(),
					});

					return Ok(().into())
				} else {
					// The item returns to chain, that sent it already, but it is not origin,
					// proceeding as normal, but removing item from sent assets
					SentAssets::<T, I>::remove(&(collection.clone(), item.clone()));
				}
			}

			// Check if the owner owns the collection
			ensure!(
				pallet_nfts::Pallet::<T, I>::collection_owner(collection).unwrap() ==
					signed_origin.clone(),
				Error::<T, I>::NotCollectionOwner
			);

			// Check if the item exists
			ensure!(
				!pallet_nfts::Item::<T, I>::contains_key(&collection, &item),
				Error::<T, I>::NFTExists
			);

			match pallet_nfts::Pallet::<T, I>::mint(
				origin.clone(),
				collection.clone(),
				item.clone(),
				signed_origin_lookup.clone(),
				None,
			) {
				Ok(_) => {},
				Err(e) => {
					// Deposit event indicating failure to mint NFT
					Self::deposit_event(Event::NFTMintFailed {
						collection_id: collection.clone(),
						asset_id: item.clone(),
						owner: signed_origin_lookup.clone(),
						error: e,
					});
				},
			}

			if !data.is_empty() {
				match pallet_nfts::Pallet::<T, I>::set_metadata(
					origin.clone(),
					collection.clone(),
					item.clone(),
					data.clone(),
				) {
					Ok(_) => {},
					Err(e) => {
						// Deposit event indicating failure to set metadata
						Self::deposit_event(Event::NFTMetadataSetFailed {
							collection_id: collection.clone(),
							asset_id: item.clone(),
							owner: signed_origin_lookup.clone(),
							error: e,
						});
					},
				}
			}

			// Check if the NFT was minted successfuly
			ensure!(
				pallet_nfts::Item::<T, I>::contains_key(&collection, &item),
				Error::<T, I>::NFTDoesNotExist
			);

			// Add the item to the received item storage
			ReceivedAssets::<T, I>::insert(
				(collection.clone(), item.clone()),
				ReceivedStruct {
					origin_para_id: origin_chain.clone(),
					origin_collection_id: origin_collection.clone(),
					origin_asset_id: origin_item.clone(),
					received_collection_id: collection.clone(),
					received_asset_id: item.clone(),
				},
			);

			// Emit a success event
			Self::deposit_event(Event::NFTReceived {
				origin_collection_id: origin_collection.clone(),
				origin_asset_id: origin_item.clone(),
				received_collection_id: collection.clone(),
				received_asset_id: item.clone(),
				to_address: signed_origin_lookup.clone(),
			});

			Ok(().into())
		}

		/// Receive function for  collection_x_transfer function.
		///
		/// Used when collection has nfts, but they are owned by the same owner.
		///
		/// Shouldn't be used as a regular call.
		///
		/// On success emits `CollectionWithNftsReceived` event.
		#[pallet::call_index(19)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_collection_same_owner(
			origin: OriginFor<T>,
			config: CollectionConfigFor<T, I>,
			collection_metadata: BoundedVec<u8, T::StringLimit>,
			nfts: Vec<(T::ItemId, BoundedVec<u8, T::StringLimit>)>,
			origin_para: ParaId,
			origin_collection_id: T::CollectionId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(who.clone());

			// Get next collection id
			let mut next_collection_id = pallet_nfts::NextCollectionId::<T, I>::get()
				.or(T::CollectionId::initial_value())
				.unwrap();

			match pallet_nfts::Pallet::<T, I>::create(
				origin.clone(),
				signed_origin_lookup.clone(),
				config.clone(),
			) {
				Ok(_) => {},
				Err(e) => {
					//  Deposit event indicating failure to create collection
					Self::deposit_event(Event::CollectionMintFailed { error: e });
				},
			}

			let mut user_collection = next_collection_id.clone();

			// just to be sure, check if user is collection owner
			if pallet_nfts::Pallet::<T, I>::collection_owner(next_collection_id.clone()).unwrap() !=
				who.clone()
			{
				//Get current next_collection_id
				let current_next_collection_id = pallet_nfts::NextCollectionId::<T, I>::get()
					.ok_or(Error::<T, I>::NoNextCollectionId)?;

				// Go from next_collection_id to current_next_collection_id and try to find the
				// collection that is owned by the user
				while next_collection_id != current_next_collection_id {
					if pallet_nfts::Pallet::<T, I>::collection_owner(next_collection_id).unwrap() ==
						who.clone()
					{
						// We have users collection
						user_collection = next_collection_id.clone();
						break;
					}
					next_collection_id =
						next_collection_id.increment().ok_or(Error::<T, I>::NoNextCollectionId)?;
				}
			}

			// Set the collection metadata if present
			if !collection_metadata.is_empty() {
				match pallet_nfts::Pallet::<T, I>::set_collection_metadata(
					origin.clone(),
					user_collection.clone(),
					collection_metadata.clone(),
				) {
					Ok(_) => {},
					Err(e) => {
						// Deposit event indicating failure to set metadata
						Self::deposit_event(Event::CollectionMetadataSetFailed {
							collection_id: user_collection.clone(),
							owner: signed_origin_lookup.clone(),
							error: e,
						});
					},
				}
			}

			//Iterate through vector of nfts
			for nft in nfts.clone() {
				let item = nft.0;
				let data = nft.1;

				match pallet_nfts::Pallet::<T, I>::mint(
					origin.clone(),
					user_collection.clone(),
					item.clone(),
					signed_origin_lookup.clone(),
					None,
				) {
					Ok(_) => {},
					Err(e) => {
						// Deposit event indicating failure to mint NFT
						Self::deposit_event(Event::NFTMintFailed {
							collection_id: user_collection.clone(),
							asset_id: item.clone(),
							owner: signed_origin_lookup.clone(),
							error: e,
						});
					},
				}
				//If empty metadata, skip
				if data.is_empty() {
					match pallet_nfts::Pallet::<T, I>::set_metadata(
						origin.clone(),
						user_collection.clone(),
						item.clone(),
						data.clone(),
					) {
						Ok(_) => {},
						Err(e) => {
							// Deposit event indicating failure to set metadata
							Self::deposit_event(Event::NFTMetadataSetFailed {
								collection_id: user_collection.clone(),
								asset_id: item.clone(),
								owner: signed_origin_lookup.clone(),
								error: e,
							});
						},
					}
				}

				// Check if the NFT was minted if storage contains the item
				ensure!(
					pallet_nfts::Item::<T, I>::contains_key(&user_collection, &item),
					Error::<T, I>::NFTDoesNotExist
				);
			}

			// Add collection to received collections
			ReceivedCollections::<T, I>::insert(
				user_collection.clone(),
				ReceivedCols {
					origin_para_id: origin_para.clone(),
					origin_collection_id: origin_collection_id.clone(),
					received_collection_id: user_collection.clone(),
				},
			);

			// Emit event about successful cross-chain operation
			Self::deposit_event(Event::CollectionWithNftsReceived {
				collection_id: user_collection.clone(),
				items: nfts.clone(),
			});

			Ok(().into())
		}

		/// Receive function for  collection_x_transfer_initiate function.
		///
		/// Used when collection has nfts, but they are not owned by the same owner.
		///
		/// Shouldn't be used as a regular call.
		///
		/// On success emits `CollectionWithNftsDiffOwnersReceived` event.
		#[pallet::call_index(20)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn parse_collection_diff_owners(
			origin: OriginFor<T>,
			config: CollectionConfigFor<T, I>,
			collection_metadata: BoundedVec<u8, T::StringLimit>,
			nfts: Vec<(T::ItemId, AccountIdLookupOf<T>, BoundedVec<u8, T::StringLimit>)>,
			origin_para: ParaId,
			origin_collection_id: T::CollectionId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			let signed_origin_lookup = T::Lookup::unlookup(who.clone());

			//Get next collection id
			let mut next_collection_id = pallet_nfts::NextCollectionId::<T, I>::get()
				.or(T::CollectionId::initial_value())
				.unwrap();

			match pallet_nfts::Pallet::<T, I>::create(
				origin.clone(),
				signed_origin_lookup.clone(),
				config.clone(),
			) {
				Ok(_) => {},
				Err(e) => {
					//  Deposit event indicating failure to create collection
					Self::deposit_event(Event::CollectionMintFailed { error: e });
				},
			}

			let mut user_collection = next_collection_id.clone();

			// just to be sure, check if user is collection owner
			if pallet_nfts::Pallet::<T, I>::collection_owner(next_collection_id.clone()).unwrap() !=
				who.clone()
			{
				//Get current next_collection_id
				let current_next_collection_id = pallet_nfts::NextCollectionId::<T, I>::get()
					.ok_or(Error::<T, I>::NoNextCollectionId)?;

				// Go from next_collection_id to current_next_collection_id and try to find the
				// collection that is owned by the user
				while next_collection_id != current_next_collection_id {
					if pallet_nfts::Pallet::<T, I>::collection_owner(next_collection_id).unwrap() ==
						who.clone()
					{
						// We have users collection
						user_collection = next_collection_id.clone();
						break;
					}
					next_collection_id =
						next_collection_id.increment().ok_or(Error::<T, I>::NoNextCollectionId)?;
				}
			}

			// Set the collection metadata if present
			if !collection_metadata.is_empty() {
				match pallet_nfts::Pallet::<T, I>::set_collection_metadata(
					origin.clone(),
					user_collection.clone(),
					collection_metadata.clone(),
				) {
					Ok(_) => {},
					Err(e) => {
						// Deposit event indicating failure to set metadata
						Self::deposit_event(Event::CollectionMetadataSetFailed {
							collection_id: user_collection.clone(),
							owner: signed_origin_lookup.clone(),
							error: e,
						});
					},
				}
			}

			//Iterate through vector of nfts
			for nft in nfts.clone() {
				let item = nft.0;
				let nft_owner = nft.1;
				let data = nft.2;

				match pallet_nfts::Pallet::<T, I>::mint(
					origin.clone(),
					user_collection.clone(),
					item.clone(),
					nft_owner.clone(),
					None,
				) {
					Ok(_) => {},
					Err(e) => {
						// Deposit event indicating failure to mint NFT
						Self::deposit_event(Event::NFTMintFailed {
							collection_id: user_collection.clone(),
							asset_id: item.clone(),
							owner: nft_owner.clone(),
							error: e,
						});
					},
				}

				if !data.is_empty() {
					match pallet_nfts::Pallet::<T, I>::set_metadata(
						origin.clone(),
						user_collection.clone(),
						item.clone(),
						data.clone(),
					) {
						Ok(_) => {},
						Err(e) => {
							// Deposit event indicating failure to set metadata
							Self::deposit_event(Event::NFTMetadataSetFailed {
								collection_id: user_collection.clone(),
								asset_id: item.clone(),
								owner: nft_owner.clone(),
								error: e,
							});
						},
					}
				}

				//Check if the NFT was minted if storage contains the item
				ensure!(
					pallet_nfts::Item::<T, I>::contains_key(&user_collection, &item),
					Error::<T, I>::NFTDoesNotExist
				);
			}

			//Add collection to received collections
			ReceivedCollections::<T, I>::insert(
				user_collection.clone(),
				ReceivedCols {
					origin_para_id: origin_para.clone(),
					origin_collection_id: origin_collection_id.clone(),
					received_collection_id: user_collection.clone(),
				},
			);

			//If all went up to this point, emit event about successful cross-chain operation
			Self::deposit_event(Event::CollectionWithNftsDiffOwnersReceived {
				collection_id: user_collection.clone(),
				items: nfts.clone(),
			});

			Ok(().into())
		}
	}
}
