use crate::{
	mock::*, Error, Event, GeneralizedDestroyWitness, Proposal, ReceivedAssets,
	ReceivedCollections, ReceivedCols, ReceivedStruct, SentAssets, SentStruct,
};

pub mod testpara;
pub mod testrelay;

use frame_support::assert_noop;
use sp_runtime::{AccountId32, BoundedVec, BuildStorage};
use cumulus_primitives_core::Parachain;
use xcm_executor::traits::ConvertLocation;
use xcm::prelude::*;
use pallet_uniques;

pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([1u8; 32]);
pub const INITIAL_BALANCE: u128 = 1_000_000_000;

use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain, TestExt};
use sp_tracing;

pub fn parent_account_id() -> testpara::AccountId {
	let location = (Parent,);
	testpara::location_converter::LocationConverter::convert_location(&location.into()).unwrap()
}

pub fn child_account_id(para: u32) -> testrelay::AccountId {
	let location = (Parachain(para),);
	testrelay::location_converter::LocationConverter::convert_location(&location.into()).unwrap()
}

decl_test_parachain! {
	pub struct ParaA {
		Runtime = testpara::Runtime,
		XcmpMessageHandler = testpara::MsgQueue,
		DmpMessageHandler = testpara::MsgQueue,
		new_ext = para_ext(1000),
	}
}

decl_test_parachain! {
	pub struct ParaB {
		Runtime = testpara::Runtime,
		XcmpMessageHandler = testpara::MsgQueue,
		DmpMessageHandler = testpara::MsgQueue,
		new_ext = para_ext(2000),
	}
}

decl_test_relay_chain! {
	pub struct Relay {
		Runtime = testrelay::Runtime,
		RuntimeCall = testrelay::RuntimeCall,
		RuntimeEvent = testrelay::RuntimeEvent,
		XcmConfig = testrelay::XcmConfig,
		MessageQueue = testrelay::MessageQueue,
		System = testrelay::System,
		new_ext = relay_ext(),
	}
}

decl_test_network! {
	pub struct MockNet {
		relay_chain = Relay,
		parachains = vec![
			(1000, ParaA),
			(2000, ParaB),
		],
	}
}



pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
	use testpara::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE), (parent_account_id(), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		sp_tracing::try_init_simple();
		System::set_block_number(1);
		MsgQueue::set_para_id(para_id.into());
	});
	ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
	use testrelay::{Runtime, RuntimeOrigin, System, NFTs};

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, INITIAL_BALANCE),
			(child_account_id(1), INITIAL_BALANCE),
			(child_account_id(2), INITIAL_BALANCE),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

#[test]
fn try_sending_collection_empty_success() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = testpara::XcNFT::collection_x_transfer(
			testpara::RuntimeOrigin::signed(ALICE),
			0,
			Some(COLLECTION_ID),
			2000.into(),
			None
		);

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CollectionTransferred { origin_collection_id: 0, origin_collection_metadata: BoundedVec::new(), destination_para_id: 2000.into()}));

	});
}

#[test]
fn try_sending_collection_same_owner_success() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0,ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 1, ALICE);

		let _ = testpara::XcNFT::collection_x_transfer(
			testpara::RuntimeOrigin::signed(ALICE),
			0,
			Some(COLLECTION_ID),
			2000.into(),
			None
		);

		let nft_ids = vec![0, 1];

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CollectionAndNFTsTransferred { origin_collection_id: 0, nft_ids, destination_para_id: 2000.into()}));

	});
}

#[test]
fn try_sending_collection_different_owners_success() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0,ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 1, BOB);

		let _ = testpara::XcNFT::collection_x_transfer(
			testpara::RuntimeOrigin::signed(ALICE),
			0,
			Some(COLLECTION_ID),
			2000.into(),
			None
		);

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CollectionTransferProposalCreated { proposal_id: 0, collection_id: 0, proposer: ALICE, destination: 2000.into()}));

	});
}

#[test]
fn try_sending_collection_that_user_doesnt_own() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		assert_noop!(
			testpara::XcNFT::collection_x_transfer(
				testpara::RuntimeOrigin::signed(BOB),
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
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);

		assert_noop!(
			testpara::XcNFT::collection_x_transfer_vote(testpara::RuntimeOrigin::signed(ALICE), 0, crate::Vote::Aye),
			Error::<Test>::ProposalDoesNotExist
		);
	});
}

#[test]
fn try_voting_on_proposal_when_no_owner() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);
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
			testpara::XcNFT::collection_x_transfer_vote(testpara::RuntimeOrigin::signed(BOB), 1, crate::Vote::Aye),
			Error::<Test>::NotNFTOwner
		);
	});
}

#[test]
fn try_voting_on_proposal_expired() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(3);
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
			testpara::XcNFT::collection_x_transfer_vote(testpara::RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye);

			testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::ProposalExpired { proposal_id: 1 }));
	});
}

#[test]
fn try_voting_on_proposal_did_not_pass() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(3);
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
			testpara::XcNFT::collection_x_transfer_vote(testpara::RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye);

			testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::ProposalDidNotPass {
			proposal_id: 1,
		}));
	});
}

#[test]
fn try_voting_on_proposal_again_same_vote() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);
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
			testpara::XcNFT::collection_x_transfer_vote(testpara::RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye);

		assert_noop!(
			testpara::XcNFT::collection_x_transfer_vote(testpara::RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye),
			Error::<Test>::AlreadyVotedThis
		);
	});
}

#[test]
fn vote_on_proposal_successfuly() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);
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
		testpara::XcNFT::collection_x_transfer_vote(testpara::RuntimeOrigin::signed(ALICE), 1, crate::Vote::Aye);

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CrossChainPropoposalVoteRegistered {
			proposal_id: 1,
			voter: ALICE,
			vote: crate::Vote::Aye,
		}));
	});
}

#[test]
fn initiate_proposal_successfuly() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);
		const COLLECTION_ID: u32 = 1;

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0,ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 1, BOB);

		let _ = testpara::XcNFT::collection_x_transfer(
			testpara::RuntimeOrigin::signed(ALICE),
			0,
			Some(COLLECTION_ID),
			2000.into(),
			None
		);

		let _ = testpara::XcNFT::collection_x_transfer_vote(testpara::RuntimeOrigin::signed(ALICE), 0, crate::Vote::Aye);

		testpara::System::set_block_number(11);

		let _ = testpara::XcNFT::collection_x_transfer_initiate(testpara::RuntimeOrigin::signed(ALICE), 0);

		let nfts = vec![(0, ALICE, BoundedVec::new()), (1, BOB, BoundedVec::new())];

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CollectionAndNFTsDiffTransferred { origin_collection_id: 0, nfts: nfts, destination_para_id: 2000.into(), to_address: ALICE }));

	});
}

#[test]
fn try_initiating_proposal_doesnt_exist() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);

		assert_noop!(
			testpara::XcNFT::collection_x_transfer_initiate(testpara::RuntimeOrigin::signed(ALICE), 1),
			Error::<Test>::ProposalDoesNotExist
		);
	});
}

#[test]
fn try_initiating_proposal_collection_doesnt_exist() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(1);

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
			testpara::XcNFT::collection_x_transfer_initiate(testpara::RuntimeOrigin::signed(ALICE), 1),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_initiating_proposal_no_collection_owner() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(BOB), 0, ALICE);

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
			testpara::XcNFT::collection_x_transfer_initiate(testpara::RuntimeOrigin::signed(ALICE), 1),
			Error::<Test>::NotCollectionOwner
		);
	});
}

#[test]
fn try_initiating_proposal_that_did_not_pass() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

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
		let _ = testpara::XcNFT::collection_x_transfer_initiate(testpara::RuntimeOrigin::signed(ALICE), 1);

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::ProposalDidNotPass { proposal_id: 1 }));
	});
}

#[test]
fn try_sending_nft_successful() {

	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0,ALICE);

		let _ = testpara::XcNFT::nft_x_transfer(testpara::RuntimeOrigin::signed(ALICE), 0, 0, 1000.into(), 1, 1);

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::NFTTransferred { origin_collection_id: 0, origin_asset_id: 0, destination_para_id: 1000.into(), destination_collection_id: 1, destination_asset_id: 1 }));
	});
}

#[test]
fn try_sending_nft_no_collection() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		assert_noop!(
			testpara::XcNFT::nft_x_transfer(testpara::RuntimeOrigin::signed(ALICE), 1, 0, 1000.into(), 1, 1),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_sending_nft_no_nft() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		assert_noop!(
			testpara::XcNFT::nft_x_transfer(testpara::RuntimeOrigin::signed(ALICE), 0, 0, 1000.into(), 1, 1),
			Error::<Test>::NFTDoesNotExist
		);
	});
}

#[test]
fn try_sending_nft_not_nft_owner() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		assert_noop!(
			testpara::XcNFT::nft_x_transfer(testpara::RuntimeOrigin::signed(BOB), 0, 0, 1000.into(), 1, 1),
			Error::<Test>::NotNFTOwner
		);
	});
}

#[test]
fn try_claiming_nft_no_collection() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		assert_noop!(
			testpara::XcNFT::nft_x_claim(testpara::RuntimeOrigin::signed(ALICE), 1u32, 0u32, 100u32.into(), 1u32, 1u32),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_claiming_nft_no_collection_origin() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		assert_noop!(
			testpara::XcNFT::nft_x_claim(testpara::RuntimeOrigin::signed(ALICE), 1u32, 0u32, 100u32.into(), 1u32, 1u32),
			Error::<Test>::CollectionDoesNotExist
		);
	});
}

#[test]
fn try_claiming_nft_wrong_origin_collection() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let collections: ReceivedCols<Test> = ReceivedCols::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 10,
			received_collection_id: 20,
		};

		let _ = ReceivedCollections::<Test>::insert(0, collections);

		assert_noop!(
			testpara::XcNFT::nft_x_claim(testpara::RuntimeOrigin::signed(ALICE), 0u32, 0u32, 100u32.into(), 0u32, 1u32),
			Error::<Test>::WrongOriginCollectionAtOrigin
		);
	});
}

#[test]
fn try_claiming_nft_wrong_nft() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);


		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let collections: ReceivedCols<Test> = ReceivedCols::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 0,
			received_collection_id: 0,
		};

		let _ = ReceivedCollections::<Test>::insert(0, collections);

		assert_noop!(
			testpara::XcNFT::nft_x_claim(testpara::RuntimeOrigin::signed(ALICE), 0u32, 0u32, 100u32.into(), 0u32, 0u32),
			Error::<Test>::NFTNotReceived
		);
	});
}

#[test]
fn try_claiming_nft_not_owner() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0u32, 0u32, ALICE);

		testpara::System::set_block_number(3);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 1, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 1u32, 0u32, ALICE);

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
			testpara::XcNFT::nft_x_claim(testpara::RuntimeOrigin::signed(BOB), 0u32, 0u32, 0u32, 1u32, 0u32),
			Error::<Test>::NotNFTOwner
		);
	});
}

#[test]
fn try_claiming_nft_success() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0u32, 0u32, ALICE);

		System::set_block_number(3);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 1, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 1u32, 0u32, ALICE);

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
		testpara::System::set_block_number(3);

		let _ = testpara::XcNFT::nft_x_claim(testpara::RuntimeOrigin::signed(ALICE), 0u32, 0u32, 0u32, 1u32, 0u32);

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::NFTClaimed {
			collection_claimed_from: 1,
			asset_removed: 0,
			collection_claimed_to: 0,
			asset_claimed: 0,
		}));
	});
}

#[test]
fn try_collection_parse_empty_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::XcNFT::parse_collection_empty(
			testpara::RuntimeOrigin::signed(ALICE),
			1,
			None,
			BoundedVec::new(),
			None,
		);

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CollectionReceived {
			origin_collection_id: 1,
			received_collection_id: 1,
			to_address: ALICE,
		}));
	});
}

#[test]
fn try_parse_collection_burn_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let destroy_witness =
			GeneralizedDestroyWitness { item_meta: 0, item_configs: 0, attributes: 0 };

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = testpara::XcNFT::parse_collection_burn(testpara::RuntimeOrigin::signed(ALICE), 0, destroy_witness);

		testpara::System::assert_has_event(testpara::RuntimeEvent::NFTs(pallet_uniques::Event::Destroyed {
			collection: 0,
		}));
	});
}

#[test]
fn try_parse_collection_metadata_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ =
		testpara::XcNFT::parse_collection_metadata(testpara::RuntimeOrigin::signed(ALICE), 0, BoundedVec::new());

		testpara::System::assert_has_event(testpara::RuntimeEvent::NFTs(pallet_uniques::Event::CollectionMetadataSet {
			collection: 0,
			data: BoundedVec::new(),
			is_frozen: false,
		}));
	});
}

#[test]
fn try_parse_nft_burn_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		let _ = testpara::XcNFT::parse_nft_burn(testpara::RuntimeOrigin::signed(ALICE), 0, 0);

		testpara::System::assert_has_event(testpara::RuntimeEvent::NFTs(pallet_uniques::Event::Burned {
			collection: 0,
			item: 0,
			owner: ALICE,
		}));
	});
}

#[test]
fn try_parse_nft_metadata_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		let _ = testpara::XcNFT::parse_nft_metadata(testpara::RuntimeOrigin::signed(ALICE), 0, 0, BoundedVec::new());

		testpara::System::assert_has_event(testpara::RuntimeEvent::NFTs(pallet_uniques::Event::MetadataSet {
			collection: 0,
			item: 0,
			data: BoundedVec::new(),
			is_frozen: false
		}));
	});
}

#[test]
fn try_parse_nft_owner_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		let _ = testpara::XcNFT::parse_nft_owner(testpara::RuntimeOrigin::signed(ALICE), BOB, 0, 0);

		testpara::System::assert_has_event(testpara::RuntimeEvent::NFTs(pallet_uniques::Event::Transferred {
			collection: 0,
			item: 0,
			from: ALICE,
			to: BOB,
		}));
	});
}

#[test]
fn try_parse_nft_transfer_no_collection() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		assert_noop!(
			testpara::XcNFT::parse_nft_transfer(
				testpara::RuntimeOrigin::signed(ALICE),
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
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		let nfts: ReceivedStruct<Test> = ReceivedStruct::<Test> {
			origin_para_id: 1000.into(),
			origin_collection_id: 0,
			origin_asset_id: 0,
			received_collection_id: 0,
			received_asset_id: 0,
		};

		let _ = ReceivedAssets::<Test>::insert((0, 0), nfts);

		assert_noop!(
			testpara::XcNFT::parse_nft_transfer(
				testpara::RuntimeOrigin::signed(ALICE),
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
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		assert_noop!(
			testpara::XcNFT::parse_nft_transfer(
				testpara::RuntimeOrigin::signed(BOB),
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
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = testpara::NFTs::mint(testpara::RuntimeOrigin::signed(ALICE), 0, 0, ALICE);

		assert_noop!(
			testpara::XcNFT::parse_nft_transfer(
				testpara::RuntimeOrigin::signed(ALICE),
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
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);
		let _ = testpara::XcNFT::parse_nft_transfer(
			testpara::RuntimeOrigin::signed(ALICE),
			0,
			0,
			BoundedVec::new(),
			0,
			0,
			1000.into(),
		);
		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::NFTReceived {
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
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::NFTs::create(testpara::RuntimeOrigin::signed(ALICE), 0, ALICE);

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

		let _ = testpara::XcNFT::parse_nft_transfer(
			testpara::RuntimeOrigin::signed(ALICE),
			0,
			0,
			BoundedVec::new(),
			0,
			0,
			ParachainInfo::parachain_id(),
		);
		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::NFTReturnedToOrigin {
			returned_from_collection_id: 0,
			returned_from_asset_id: 0,
			to_address: ALICE,
		}));
	});
}

#[test]
fn parse_collection_same_owner_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let mut nfts: Vec<(u32, BoundedVec<u8, testpara::UniquesStringLimit>)> = Vec::new();
		nfts.push((1, BoundedVec::new()));

		let _ = testpara::XcNFT::parse_collection_same_owner(
			testpara::RuntimeOrigin::signed(ALICE),
			None,
			BoundedVec::new(),
			nfts.clone(),
			1000.into(),
			0,
			None,
		);
		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CollectionWithNftsReceived {
			collection_id: 0,
			items: nfts.clone(),
		}));
	});
}

#[test]
fn parse_collection_diff_nft_owners_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let mut nfts: Vec<(u32, AccountId32, BoundedVec<u8, testpara::UniquesStringLimit>)> = Vec::new();
		nfts.push((1, BOB, BoundedVec::new()));

		let _ = testpara::XcNFT::parse_collection_diff_owners(
			testpara::RuntimeOrigin::signed(ALICE),
			None,
			BoundedVec::new(),
			nfts.clone(),
			1000.into(),
			0,
			None,
		);
		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(
			Event::CollectionWithNftsDiffOwnersReceived { collection_id: 0, items: nfts.clone() },
		));
	});
}

#[test]
fn try_collection_metadata_success(){
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::XcNFT::collection_x_update(testpara::RuntimeOrigin::signed(ALICE), 0, 1000.into(), BoundedVec::new());

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CollectionMetadataSent { collection_id: 0, proposed_data: BoundedVec::new(), owner: ALICE, destination: 1000.into()}));
	});
}

#[test]
fn try_collection_owner_send_success(){
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::XcNFT::collection_x_change_owner(testpara::RuntimeOrigin::signed(ALICE), 0, 1000.into(), BOB);

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CollectionOwnershipSent { collection_id: 0, proposed_owner: BOB, destination: 1000.into() }));
	});
}

#[test]
fn try_collection_burn_success(){
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let witness = GeneralizedDestroyWitness { item_meta: 0, item_configs: 0, attributes: 0 };

		let _ = testpara::XcNFT::collection_x_burn(testpara::RuntimeOrigin::signed(ALICE), 0, 1000.into(), witness.clone());

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::CollectionBurnSent { collection_id: 0, burn_data: witness.clone(), owner: ALICE, destination: 1000.into() }));
	});
}

#[test]
fn try_nft_metadata_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::XcNFT::nft_x_update(testpara::RuntimeOrigin::signed(ALICE), 0, 0, 1000.into(), BoundedVec::new());

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::NFTMetadataSent { collection_id: 0, asset_id: 0, proposed_data: BoundedVec::new(), owner: ALICE, destination: 1000.into() }));
	});
}

#[test]
fn try_nft_owner_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::XcNFT::nft_x_change_owner(testpara::RuntimeOrigin::signed(ALICE), 0, 0, 1000.into(), BOB);

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::NFTOwnershipSent { collection_id: 0, asset_id: 0, proposed_owner: BOB, destination: 1000.into() }));
	});
}

#[test]
fn try_nft_burn_successful() {
	ParaA::execute_with(|| {
		testpara::System::set_block_number(2);

		let _ = testpara::XcNFT::nft_x_burn(testpara::RuntimeOrigin::signed(ALICE), 0, 0, 1000.into());

		testpara::System::assert_has_event(testpara::RuntimeEvent::XcNFT(Event::NFTBurnSent { collection_id: 0, asset_id: 0, owner: ALICE, destination: 1000.into() }));
	});
}