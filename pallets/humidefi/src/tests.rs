use crate::{mock::{self, *}, AssetPairs, Error, Event};
use frame_support::{assert_noop, assert_ok};
use frame_support::{
	sp_runtime::{
		traits::{Convert, EnsureAdd, EnsureDiv, EnsureMul, EnsureSub, IntegerSquareRoot, Zero},
		ArithmeticError, FixedU128, Perbill,
	},
	traits::{fungible, fungibles},
};
use interfaces::humidefi::HumidefiHelpers;

#[test]
fn create_liquidity_pool_works() {
	new_test_ext().execute_with(|| {
		let bob = RuntimeOrigin::signed(2);
		let first_balance_a: u128 = 15_000_000_000_000_000_000_000_000;
		let first_balance_b: u128 = 20_000_000_000_000_000_000_000_000;

		let asset_pairs = AssetPairs {
			asset_x: 1,
			asset_y: 2
		};

		assert_ok!(Humidefi::new_liquidity(bob.clone(), asset_pairs, first_balance_a, first_balance_b));
	});
}
