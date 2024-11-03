use crate::{
	mock::*, Error, Event, GeneralizedDestroyWitness, Proposal, ReceivedAssets,
	ReceivedCollections, ReceivedCols, ReceivedStruct, SentAssets, SentStruct,
};
use frame_support::assert_noop;
use pallet_uniques::Event::Destroyed;
use sp_runtime::{AccountId32, BoundedVec};

pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([1u8; 32]);

#[test]
fn try_sending_collection_that_doesnt_exist() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		assert_noop!(
			XcNFT::collection_x_transfer(
				RuntimeOrigin::signed(ALICE),
				COLLECTION_ID,
				Some(COLLECTION_ID),
				2000.into(),
				None
			),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_sending_collection_that_user_doesnt_own() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		assert_noop!(
			XcNFT::collection_x_transfer(
				RuntimeOrigin::signed(BOB),
				0,
				Some(COLLECTION_ID),
				2000.into(),
				None
			),
			Error::<Test>::NotCollectionOwner
		);
	});
}

#[test]
fn try_voting_on_non_existing_proposal() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			XcNFT::collection_x_transfer_vote(RuntimeOrigin::signed(ALICE), 0, crate::Vote::Aye),
			Error::<Test>::ProposalDoesNotExist
		);
	});
}

#[test]
fn try_voting_on_proposal_when_no_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		//Create proposal
		let proposal = Proposal::<Test> {
			proposal_id: 1,
			collection_id: COLLECTION_ID,
			proposed_collection_owner: ALICE,
			proposed_destination_para: 2000.into(),
			proposed_dest_collection_id: None,
			proposed_destination_config: None,
			owners: BoundedVec::new(),
			number_of_votes: crate::Votes { aye: BoundedVec::new(), nay: BoundedVec::new() },
			end_time: 20u64.into(),
		};

		let _ = crate::CrossChainProposals::insert(1, proposal);

		assert_noop!(
			XcNFT::collection_x_transfer_vote(RuntimeOrigin::signed(BOB), 1, crate::Vote::Aye),
			Error::<Test>::NotNFTOwner
		);
	});
}

#[test]
fn try_voting_on_proposal_expired() {
	new_test_ext().execute_with(|| {
		System::set_block_number(3);
		const COLLECTION_ID: u32 = 1;

		//Create owners vector
		let mut owners = BoundedVec::new();
		owners.try_push(ALICE).expect("Failed to push owner");

		//Create proposal
		let proposal = Proposal::<Test> {
			proposal_id: 1,
			collection_id: COLLECTION_ID,
			proposed_collection_owner: ALICE,
			proposed_destination_para: 2000.into(),
			proposed_dest_collection_id: None,
			proposed_destination_config: None,
			owners: owners.clone(),
			number_of_votes: crate::Votes { aye: owners.clone(), nay: BoundedVec::new() },
			end_time: 1u64.into(),
		};

		let _ = crate::CrossChainProposals::insert(1, proposal);

		let _ =
			XcNFT::collection_x_transfer_vote(RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye);

		System::assert_last_event(RuntimeEvent::XcNFT(Event::ProposalExpired { proposal_id: 1 }));
	});
}

#[test]
fn try_voting_on_proposal_did_not_pass() {
	new_test_ext().execute_with(|| {
		System::set_block_number(3);
		const COLLECTION_ID: u32 = 1;

		//Create owners vector
		let mut owners = BoundedVec::new();
		owners.try_push(ALICE).expect("Failed to push owner");

		//Create proposal
		let proposal = Proposal::<Test> {
			proposal_id: 1,
			collection_id: COLLECTION_ID,
			proposed_collection_owner: ALICE,
			proposed_destination_para: 2000.into(),
			proposed_dest_collection_id: None,
			proposed_destination_config: None,
			owners: owners.clone(),
			number_of_votes: crate::Votes { aye: BoundedVec::new(), nay: owners.clone() },
			end_time: 1u64.into(),
		};

		let _ = crate::CrossChainProposals::insert(1, proposal);

		let _ =
			XcNFT::collection_x_transfer_vote(RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye);

		System::assert_last_event(RuntimeEvent::XcNFT(Event::ProposalDidNotPass {
			proposal_id: 1,
		}));
	});
}

#[test]
fn try_voting_on_proposal_again_same_vote() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		//Create owners vector
		let mut owners = BoundedVec::new();
		owners.try_push(ALICE).expect("Failed to push owner");

		//Create proposal
		let proposal = Proposal::<Test> {
			proposal_id: 1,
			collection_id: COLLECTION_ID,
			proposed_collection_owner: ALICE,
			proposed_destination_para: 2000.into(),
			proposed_dest_collection_id: None,
			proposed_destination_config: None,
			owners: owners.clone(),
			number_of_votes: crate::Votes { aye: BoundedVec::new(), nay: owners.clone() },
			end_time: 3u64.into(),
		};

		let _ = crate::CrossChainProposals::insert(1, proposal);

		let _ =
			XcNFT::collection_x_transfer_vote(RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye);

		assert_noop!(
			XcNFT::collection_x_transfer_vote(RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye),
			Error::<Test>::AlreadyVotedThis
		);
	});
}

#[test]
fn vote_on_proposal_successfuly() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		//Create owners vector
		let mut owners = BoundedVec::new();
		owners.try_push(ALICE).expect("Failed to push owner");

		//Create proposal
		let proposal = Proposal::<Test> {
			proposal_id: 1,
			collection_id: COLLECTION_ID,
			proposed_collection_owner: ALICE,
			proposed_destination_para: 2000.into(),
			proposed_dest_collection_id: None,
			proposed_destination_config: None,
			owners: owners.clone(),
			number_of_votes: crate::Votes { aye: BoundedVec::new(), nay: owners.clone() },
			end_time: 2u64.into(),
		};

		let _ = crate::CrossChainProposals::insert(1, proposal);

		let _ =
			XcNFT::collection_x_transfer_vote(RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye);

		System::assert_last_event(RuntimeEvent::XcNFT(Event::CrossChainPropoposalVoteRegistered {
			proposal_id: 1,
			voter: ALICE,
			vote: crate::Vote::Aye,
		}));
	});
}

#[test]
fn try_initiating_proposal_doesnt_exist() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_noop!(
			XcNFT::collection_x_transfer_initiate(RuntimeOrigin::signed(ALICE), 1),
			Error::<Test>::ProposalDoesNotExist
		);
	});
}

#[test]
fn try_initiating_proposal_collection_doesnt_exist() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		const COLLECTION_ID: u32 = 1;

		//Create owners vector
		let mut owners = BoundedVec::new();
		owners.try_push(ALICE).expect("Failed to push owner");

		//Create proposal
		let proposal = Proposal::<Test> {
			proposal_id: 1,
			collection_id: COLLECTION_ID,
			proposed_collection_owner: ALICE,
			proposed_destination_para: 2000.into(),
			proposed_dest_collection_id: None,
			proposed_destination_config: None,
			owners: owners.clone(),
			number_of_votes: crate::Votes { aye: BoundedVec::new(), nay: owners.clone() },
			end_time: 2u64.into(),
		};

		let _ = crate::CrossChainProposals::insert(1, proposal);

		assert_noop!(
			XcNFT::collection_x_transfer_initiate(RuntimeOrigin::signed(ALICE), 1),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_initiating_proposal_no_collection_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);
		let _ = Uniques::create(RuntimeOrigin::signed(BOB), 0, ALICE);

		//Create owners vector
		let mut owners = BoundedVec::new();
		owners.try_push(BOB).expect("Failed to push owner");

		//Create proposal
		let proposal = Proposal::<Test> {
			proposal_id: 1,
			collection_id: 0,
			proposed_collection_owner: BOB,
			proposed_destination_para: 2000.into(),
			proposed_dest_collection_id: None,
			proposed_destination_config: None,
			owners: owners.clone(),
			number_of_votes: crate::Votes { aye: BoundedVec::new(), nay: owners.clone() },
			end_time: 1u64.into(),
		};

		let _ = crate::CrossChainProposals::insert(1, proposal);

		assert_noop!(
			XcNFT::collection_x_transfer_initiate(RuntimeOrigin::signed(ALICE), 1),
			Error::<Test>::NotCollectionOwner
		);
	});
}

#[test]
fn try_initiating_proposal_that_did_not_pass() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		//Create owners vector
		let mut owners = BoundedVec::new();
		owners.try_push(ALICE).expect("Failed to push owner");

		//Create proposal
		let proposal = Proposal::<Test> {
			proposal_id: 1,
			collection_id: 0,
			proposed_collection_owner: ALICE,
			proposed_destination_para: 2000.into(),
			proposed_dest_collection_id: None,
			proposed_destination_config: None,
			owners: owners.clone(),
			number_of_votes: crate::Votes { aye: BoundedVec::new(), nay: owners.clone() },
			end_time: 1u64.into(),
		};

		let _ = crate::CrossChainProposals::insert(1, proposal);
		let _ = XcNFT::collection_x_transfer_initiate(RuntimeOrigin::signed(ALICE), 1);

		System::assert_has_event(RuntimeEvent::XcNFT(Event::ProposalDidNotPass { proposal_id: 1 }));
	});
}

#[test]
fn try_sending_nft_no_collection() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		assert_noop!(
			XcNFT::nft_x_transfer(RuntimeOrigin::signed(ALICE), 1, 0, 1000.into(), 1, 1),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_sending_nft_no_nft() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		assert_noop!(
			XcNFT::nft_x_transfer(RuntimeOrigin::signed(ALICE), 0, 0, 1000.into(), 1, 1),
			Error::<Test>::NFTDoesNotExist
		);
	});
}

#[test]
fn try_sending_nft_not_nft_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		assert_noop!(
			XcNFT::nft_x_transfer(RuntimeOrigin::signed(BOB), 0, 0, 1000.into(), 1, 1),
			Error::<Test>::NotNFTOwner
		);
	});
}

#[test]
fn try_claiming_nft_no_collection() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		assert_noop!(
			XcNFT::nft_x_claim(RuntimeOrigin::signed(ALICE), 1u32, 0u32, 100u32.into(), 1u32, 1u32),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_claiming_nft_no_collection_origin() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		assert_noop!(
			XcNFT::nft_x_claim(RuntimeOrigin::signed(ALICE), 1u32, 0u32, 100u32.into(), 1u32, 1u32),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_claiming_nft_wrong_origin_collection() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let collections: ReceivedCols<Test> = ReceivedCols::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 10,
			received_collection_id: 20,
		};

		let _ = ReceivedCollections::<Test>::insert(0, collections);

		assert_noop!(
			XcNFT::nft_x_claim(RuntimeOrigin::signed(ALICE), 0u32, 0u32, 100u32.into(), 0u32, 1u32),
			Error::<Test>::WrongOriginCollectionAtOrigin
		);
	});
}

#[test]
fn try_claiming_nft_wrong_nft() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let collections: ReceivedCols<Test> = ReceivedCols::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 0,
			received_collection_id: 0,
		};

		let _ = ReceivedCollections::<Test>::insert(0, collections);

		assert_noop!(
			XcNFT::nft_x_claim(RuntimeOrigin::signed(ALICE), 0u32, 0u32, 100u32.into(), 0u32, 0u32),
			Error::<Test>::NFTNotReceived
		);
	});
}

#[test]
fn try_claiming_nft_not_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 0u32, 0u32, ALICE);

		System::set_block_number(3);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 1, ALICE);
		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 1u32, 0u32, ALICE);

		let collections: ReceivedCols<Test> = ReceivedCols::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 0,
			received_collection_id: 0,
		};

		let _ = ReceivedCollections::<Test>::insert(0, collections);

		let nfts: ReceivedStruct<Test> = ReceivedStruct::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 0,
			origin_asset_id: 0,
			received_collection_id: 1,
			received_asset_id: 0,
		};

		let _ = ReceivedAssets::<Test>::insert((1, 0), nfts);

		assert_noop!(
			XcNFT::nft_x_claim(RuntimeOrigin::signed(BOB), 0u32, 0u32, 0u32, 1u32, 0u32),
			Error::<Test>::NotNFTOwner
		);
	});
}

#[test]
fn try_claiming_nft_success() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 0u32, 0u32, ALICE);

		System::set_block_number(3);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 1, ALICE);
		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 1u32, 0u32, ALICE);

		let collections: ReceivedCols<Test> = ReceivedCols::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 0,
			received_collection_id: 0,
		};

		let _ = ReceivedCollections::<Test>::insert(0, collections);

		let nfts: ReceivedStruct<Test> = ReceivedStruct::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 0,
			origin_asset_id: 0,
			received_collection_id: 1,
			received_asset_id: 0,
		};

		let _ = ReceivedAssets::<Test>::insert((1, 0), nfts);
		System::set_block_number(3);

		let _ = XcNFT::nft_x_claim(RuntimeOrigin::signed(ALICE), 0u32, 0u32, 0u32, 1u32, 0u32);

		System::assert_has_event(RuntimeEvent::XcNFT(Event::NFTClaimed {
			collection_claimed_from: 1,
			asset_removed: 0,
			collection_claimed_to: 0,
			asset_claimed: 0,
		}));
	});
}

#[test]
fn try_collection_parse_empty_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = XcNFT::parse_collection_empty(
			RuntimeOrigin::signed(ALICE),
			1,
			None,
			BoundedVec::new(),
			None,
		);

		System::assert_has_event(RuntimeEvent::XcNFT(Event::CollectionReceived {
			origin_collection_id: 1,
			received_collection_id: 1,
			to_address: ALICE,
		}));
	});
}

#[test]
fn try_parse_collection_burn_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let destroy_witness =
			GeneralizedDestroyWitness { item_meta: 0, item_configs: 0, attributes: 0 };

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = XcNFT::parse_collection_burn(RuntimeOrigin::signed(ALICE), 0, destroy_witness);

		System::assert_has_event(RuntimeEvent::Uniques(pallet_uniques::Event::Destroyed {
			collection: 0,
		}));
	});
}

#[test]
fn try_parse_collection_metadata_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ =
			XcNFT::parse_collection_metadata(RuntimeOrigin::signed(ALICE), 0, BoundedVec::new());

		System::assert_has_event(RuntimeEvent::Uniques(
			pallet_uniques::Event::CollectionMetadataSet {
				collection: 0,
				data: BoundedVec::new(),
				is_frozen: false,
			},
		));
	});
}

#[test]
fn try_parse_collection_owner_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		pallet_uniques::OwnershipAcceptance::<Test>::insert(BOB, 0);

		let _ = XcNFT::parse_collection_owner(RuntimeOrigin::signed(ALICE), BOB, 0);

		System::assert_has_event(RuntimeEvent::Uniques(pallet_uniques::Event::OwnerChanged {
			collection: 0,
			new_owner: BOB,
		}));
	});
}

#[test]
fn try_parse_nft_burn_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		let _ = XcNFT::parse_nft_burn(RuntimeOrigin::signed(ALICE), 0, 0);

		System::assert_has_event(RuntimeEvent::Uniques(pallet_uniques::Event::Burned {
			collection: 0,
			item: 0,
			owner: ALICE,
		}));
	});
}

#[test]
fn try_parse_nft_metadata_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		let _ = XcNFT::parse_nft_metadata(RuntimeOrigin::signed(ALICE), 0, 0, BoundedVec::new());

		System::assert_has_event(RuntimeEvent::Uniques(pallet_uniques::Event::MetadataSet {
			collection: 0,
			item: 0,
			data: BoundedVec::new(),
			is_frozen: false,
		}));
	});
}

#[test]
fn try_parse_nft_owner_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		let _ = XcNFT::parse_nft_owner(RuntimeOrigin::signed(ALICE), BOB, 0, 0);

		System::assert_has_event(RuntimeEvent::Uniques(pallet_uniques::Event::Transferred {
			collection: 0,
			item: 0,
			from: ALICE,
			to: BOB,
		}));
	});
}

#[test]
fn try_parse_nft_transfer_no_collection() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		assert_noop!(
			XcNFT::parse_nft_transfer(
				RuntimeOrigin::signed(ALICE),
				0,
				0,
				BoundedVec::new(),
				0,
				0,
				1000.into()
			),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_parse_nft_transfer_already_received() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		let nfts: ReceivedStruct<Test> = ReceivedStruct::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 0,
			origin_asset_id: 0,
			received_collection_id: 0,
			received_asset_id: 0,
		};

		let _ = ReceivedAssets::<Test>::insert((0, 0), nfts);

		assert_noop!(
			XcNFT::parse_nft_transfer(
				RuntimeOrigin::signed(ALICE),
				0,
				0,
				BoundedVec::new(),
				0,
				0,
				1000.into()
			),
			Error::<Test>::NFTAlreadyReceived
		);
	});
}

#[test]
fn try_parse_nft_transfer_not_collection_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		assert_noop!(
			XcNFT::parse_nft_transfer(
				RuntimeOrigin::signed(BOB),
				0,
				0,
				BoundedVec::new(),
				0,
				0,
				1000.into()
			),
			Error::<Test>::NotCollectionOwner
		);
	});
}

#[test]
fn try_parse_nft_transfer_not_existing_nft() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = Uniques::mint(RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		assert_noop!(
			XcNFT::parse_nft_transfer(
				RuntimeOrigin::signed(ALICE),
				0,
				0,
				BoundedVec::new(),
				0,
				0,
				1000.into()
			),
			Error::<Test>::NFTExists
		);
	});
}

#[test]
fn try_parse_nft_transfer_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = XcNFT::parse_nft_transfer(
			RuntimeOrigin::signed(ALICE),
			0,
			0,
			BoundedVec::new(),
			0,
			0,
			1000.into(),
		);
		System::assert_has_event(RuntimeEvent::XcNFT(Event::NFTReceived {
			origin_collection_id: 0,
			origin_asset_id: 0,
			received_collection_id: 0,
			received_asset_id: 0,
			to_address: ALICE,
		}));
	});
}

#[test]
fn try_parse_nft_transfer_return_to_origin() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let _ = Uniques::create(RuntimeOrigin::signed(ALICE), 0, ALICE);

		let sent = SentStruct::<Test> {
			origin_para_id: ParachainInfo::parachain_id(),
			origin_collection_id: 0,
			origin_asset_id: 0,
			destination_collection_id: 0,
			destination_asset_id: 0,
		};

		let _ = SentAssets::<Test>::insert((0, 0), sent);

		//Set parachain id to 1000
		ParachainInfo::parachain_id();

		let _ = XcNFT::parse_nft_transfer(
			RuntimeOrigin::signed(ALICE),
			0,
			0,
			BoundedVec::new(),
			0,
			0,
			ParachainInfo::parachain_id(),
		);
		System::assert_has_event(RuntimeEvent::XcNFT(Event::NFTReturnedToOrigin {
			returned_from_collection_id: 0,
			returned_from_asset_id: 0,
			to_address: ALICE,
		}));
	});
}

#[test]
fn parse_collection_same_owner_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let mut nfts: Vec<(u32, BoundedVec<u8, UniquesStringLimit>)> = Vec::new();
		nfts.push((1, BoundedVec::new()));

		let _ = XcNFT::parse_collection_same_owner(
			RuntimeOrigin::signed(ALICE),
			None,
			BoundedVec::new(),
			nfts.clone(),
			1000.into(),
			0,
			None,
		);
		System::assert_has_event(RuntimeEvent::XcNFT(Event::CollectionWithNftsReceived {
			collection_id: 0,
			items: nfts.clone(),
		}));
	});
}

#[test]
fn parse_collection_diff_nft_owners_successful() {
	new_test_ext().execute_with(|| {
		System::set_block_number(2);

		let mut nfts: Vec<(u32, AccountId32, BoundedVec<u8, UniquesStringLimit>)> = Vec::new();
		nfts.push((1, BOB, BoundedVec::new()));

		let _ = XcNFT::parse_collection_diff_owners(
			RuntimeOrigin::signed(ALICE),
			None,
			BoundedVec::new(),
			nfts.clone(),
			1000.into(),
			0,
			None,
		);
		System::assert_has_event(RuntimeEvent::XcNFT(
			Event::CollectionWithNftsDiffOwnersReceived { collection_id: 0, items: nfts.clone() },
		));
	});
}
