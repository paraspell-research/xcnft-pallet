use crate::{mock::*, Error, Event};
use cumulus_primitives_core::SendError::NotApplicable;
use frame_support::assert_noop;
use sp_core::H256;

#[test]
fn try_mint_collection_transfer_xcm_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		let _ = XnftModule::mint_collection_xtransfer(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			2000.into(),
			1,
		);
		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		System::assert_last_event(
			Event::CollectionFailedToXCM {
				e: NotApplicable,
				collection_hash,
				owner: 1,
				destination: 2000.into(),
			}
			.into(),
		);
	});
}

#[test]
fn try_mint_collection_transfer_collection_exists_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//Let's mint a collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		//Lets create hash to compare
		let _collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Try creating new collection
		assert_noop!(
			XnftModule::mint_collection_xtransfer(
				RuntimeOrigin::signed(1),
				b"Collection".to_vec().try_into().unwrap(),
				b"Description".to_vec().try_into().unwrap(),
				2000.into(),
				1,
			),
			Error::<Test>::CollectionAlreadyExists
		);
	});
}

#[test]
fn try_receive_existing_collection() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//Let's mint a collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		//Try receiving new collection
		let _ = XnftModule::mint_collection_received(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			1000.into(),
			1,
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		System::assert_last_event(Event::CollectionAlreadyExistsOnChain { collection_hash }.into());
	});
}

#[test]
fn try_receive_new_collection() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//Let's receive a collection
		let _ = XnftModule::mint_collection_received(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			1000.into(),
			1,
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Try receiving new collection
		System::assert_last_event(
			Event::CollectionReceived { collection_hash, owner: 1, origin: 1000.into() }.into(),
		);
	});
}

#[test]
fn try_mint_collection_existing() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//Let's mint a collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		//Try creating new collection
		assert_noop!(
			XnftModule::mint_collection(
				RuntimeOrigin::signed(1),
				b"Collection".to_vec().try_into().unwrap(),
				b"Description".to_vec().try_into().unwrap(),
			),
			Error::<Test>::CollectionAlreadyExists
		);
	});
}

#[test]
fn try_mint_collection() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//Let's mint a collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Try creating new collection
		System::assert_last_event(Event::CollectionMinted { collection_hash, owner: 1 }.into());
	});
}

#[test]
fn try_collection_transfer_xcm_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		let _ = XnftModule::collection_xtransfer(
			RuntimeOrigin::signed(1),
			collection_hash,
			2000.into(),
			1,
		);

		System::assert_last_event(
			Event::CollectionFailedToXCM {
				e: NotApplicable,
				collection_hash,
				owner: 1,
				destination: 2000.into(),
			}
			.into(),
		);
	});
}

#[test]
fn try_collection_transfer_non_existing_collection() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//Try creating new collection
		assert_noop!(
			XnftModule::collection_xtransfer(
				RuntimeOrigin::signed(1),
				H256::zero(),
				2000.into(),
				1,
			),
			Error::<Test>::InvalidCollection
		);
	});
}

#[test]
fn try_collection_transfer_with_no_permission() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//Let's mint a collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Send without permission
		assert_noop!(
			XnftModule::collection_xtransfer(
				RuntimeOrigin::signed(2),
				collection_hash,
				2000.into(),
				1,
			),
			Error::<Test>::Unauthorized
		);
	});
}

#[test]
fn mint_nft_xtransfer_collection_not_existing() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		assert_noop!(
			XnftModule::mint_non_fungible_xtransfer(
				RuntimeOrigin::signed(1),
				b"NFT".to_vec().try_into().unwrap(),
				b"Description".to_vec().try_into().unwrap(),
				collection_hash,
				2000.into(),
				1000.into(),
				1,
			),
			Error::<Test>::InvalidCollection
		);
	});
}

#[test]
fn nft_mint_collection_not_existing() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		assert_noop!(
			XnftModule::mint_non_fungible(
				RuntimeOrigin::signed(1),
				b"NFT".to_vec().try_into().unwrap(),
				b"Description".to_vec().try_into().unwrap(),
				collection_hash,
			),
			Error::<Test>::InvalidCollection
		);
	});
}

#[test]
fn mint_non_fungible_unauthorized() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//Let's mint a collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Send without permission
		assert_noop!(
			XnftModule::mint_non_fungible(
				RuntimeOrigin::signed(2),
				b"NFT".to_vec().try_into().unwrap(),
				b"Description".to_vec().try_into().unwrap(),
				collection_hash,
			),
			Error::<Test>::Unauthorized
		);
	});
}

#[test]
fn mint_non_fungible_collection_full() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//mint collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Mint collection in for cycle
		for i in 0..255 {
			let description = format!("Hi {}", i);
			let bytes_desc = description.as_bytes();
			let _ = XnftModule::mint_non_fungible(
				RuntimeOrigin::signed(1),
				b"NFT".to_vec().try_into().unwrap(),
				bytes_desc.to_vec().try_into().unwrap(),
				collection_hash,
			);
		}

		//Try minting one more
		assert_noop!(
			XnftModule::mint_non_fungible(
				RuntimeOrigin::signed(1),
				b"NFT".to_vec().try_into().unwrap(),
				b"Hola".to_vec().try_into().unwrap(),
				collection_hash,
			),
			Error::<Test>::CollectionFull
		);
	});
}

#[test]
fn mint_non_fungible_already_exists() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//mint collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Mint collection in for cycle
		let _ = XnftModule::mint_non_fungible(
			RuntimeOrigin::signed(1),
			b"NFT".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			collection_hash,
		);

		//Try minting one more
		assert_noop!(
			XnftModule::mint_non_fungible(
				RuntimeOrigin::signed(1),
				b"NFT".to_vec().try_into().unwrap(),
				b"Description".to_vec().try_into().unwrap(),
				collection_hash,
			),
			Error::<Test>::NonFungibleAlreadyExists
		);
	});
}

#[test]
fn mint_non_fungible_success() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//mint collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Mint collection in for cycle
		let _ = XnftModule::mint_non_fungible(
			RuntimeOrigin::signed(1),
			b"NFT".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			collection_hash,
		);

		let non_fungible_hash: H256 =
			"0xe23ba5b8b703c74cf4c1da0245d4d7a69074e8d283c35f80efbe38a07dc34870"
				.parse()
				.unwrap();

		//Try minting one more
		System::assert_last_event(
			Event::NonFungibleMinted { nft_hash: non_fungible_hash, owner: 1 }.into(),
		);
	});
}

#[test]
fn mint_non_fungible_received_to_non_existing_collection() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);
		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 2000);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		let _ = XnftModule::mint_non_fungible_received(
			RuntimeOrigin::signed(1),
			b"NFT".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			collection_hash,
			2000.into(),
			1,
		);

		System::assert_last_event(Event::InvalidReceivingCollection { collection_hash }.into());
	});
}

#[test]
fn mint_non_fungible_received_collection_full() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);
		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 2000);

		//mint collection
		let _ = XnftModule::mint_collection_received(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			1000.into(),
			1,
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Mint collection in for cycle
		for i in 0..257 {
			let description = format!("Hi {}", i);
			let bytes_desc = description.as_bytes();
			let _ = XnftModule::mint_non_fungible_received(
				RuntimeOrigin::signed(1),
				b"NFT".to_vec().try_into().unwrap(),
				bytes_desc.to_vec().try_into().unwrap(),
				collection_hash,
				1000.into(),
				1,
			);
		}

		System::assert_last_event(Event::ReceivingCollectionFull { collection_hash }.into());
	});
}

#[test]
fn mint_non_fungible_received_already_exists() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);
		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 2000);

		//mint collection
		let _ = XnftModule::mint_collection_received(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			1000.into(),
			1,
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		let _ = XnftModule::mint_non_fungible_received(
			RuntimeOrigin::signed(1),
			b"NFT".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			collection_hash,
			1000.into(),
			1,
		);

		let _ = XnftModule::mint_non_fungible_received(
			RuntimeOrigin::signed(1),
			b"NFT".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			collection_hash,
			1000.into(),
			1,
		);

		let nft_hash: H256 = "0xe23ba5b8b703c74cf4c1da0245d4d7a69074e8d283c35f80efbe38a07dc34870"
			.parse()
			.unwrap();

		//Try minting one more
		System::assert_last_event(Event::NonFungibleAlreadyExisting { nft_hash }.into());
	});
}

#[test]
fn mint_non_fungible_received_success() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);
		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 2000);

		//mint collection
		let _ = XnftModule::mint_collection_received(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			1000.into(),
			1,
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//Mint collection in for cycle
		let _ = XnftModule::mint_non_fungible_received(
			RuntimeOrigin::signed(1),
			b"NFT".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			collection_hash,
			1000.into(),
			1,
		);

		let non_fungible_hash: H256 =
			"0xe23ba5b8b703c74cf4c1da0245d4d7a69074e8d283c35f80efbe38a07dc34870"
				.parse()
				.unwrap();

		//Try minting one more
		System::assert_last_event(
			Event::NonFungibleMinted { nft_hash: non_fungible_hash, owner: 1 }.into(),
		);
	});
}

#[test]
fn non_fungible_xtransfer_not_existing_nft() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//try minting sending not existing nft
		assert_noop!(
			XnftModule::non_fungible_xtransfer(
				RuntimeOrigin::signed(1),
				H256::zero(),
				2000.into(),
				1000.into(),
				1,
			),
			Error::<Test>::InvalidNonFungible
		);
	});
}

#[test]
fn non_fungible_xtransfer_unauthorized() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//mint collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//mint non fungible
		let _ = XnftModule::mint_non_fungible(
			RuntimeOrigin::signed(1),
			b"NFT".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			collection_hash,
		);

		let nft_hash = "0xe23ba5b8b703c74cf4c1da0245d4d7a69074e8d283c35f80efbe38a07dc34870"
			.parse()
			.unwrap();

		//Send without permission
		assert_noop!(
			XnftModule::non_fungible_xtransfer(
				RuntimeOrigin::signed(2),
				nft_hash,
				2000.into(),
				1000.into(),
				1,
			),
			Error::<Test>::Unauthorized
		);
	});
}

#[test]
fn non_fungible_xtransfer_not_sent() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		//mint collection
		let _ = XnftModule::mint_collection(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
		);

		let collection_hash: H256 =
			"0x06fa4188b7fa31e8b2d21dc94819e22684f9e2e3995f2ca3716404e2df6b3cf0"
				.parse()
				.unwrap();

		//mint non fungible
		let _ = XnftModule::mint_non_fungible(
			RuntimeOrigin::signed(1),
			b"NFT".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			collection_hash,
		);

		let nft_hash = "0xe23ba5b8b703c74cf4c1da0245d4d7a69074e8d283c35f80efbe38a07dc34870"
			.parse()
			.unwrap();

		//Send without permission
		assert_noop!(
			XnftModule::non_fungible_xtransfer(
				RuntimeOrigin::signed(1),
				nft_hash,
				2000.into(),
				1000.into(),
				1,
			),
			Error::<Test>::CollectionIsNotSentCrossChain
		);
	});
}

#[test]
fn tokens_deposited() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		System::assert_last_event(Event::TokensDeposited { who: 1, amount: 1000 }.into());
	});
}
