use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;

#[test]
fn try_mint_collection_transfer() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let _ = XnftModule::deposit_token(RuntimeOrigin::root(), 1, 1000);

		let _ = XnftModule::MintCollectionXTransfer(
			RuntimeOrigin::signed(1),
			b"Collection".to_vec().try_into().unwrap(),
			b"Description".to_vec().try_into().unwrap(),
			2000,
		);
		let proposal_hash: H256 =
			"0x3f5bf665dbeaf7bf8b6f98cb323b2c62e5faa9f2de8ca9c33a31995b687e7968"
				.parse()
				.unwrap();

		System::assert_last_event(
			Event::CollectionCreatedAndTransferedXCM {
				collection_hash: proposal_hash,
				owner: 1,
				destination: 2000,
			}
			.into(),
		);
	});
}
