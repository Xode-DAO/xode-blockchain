use crate::{mock::{self, *}, AssetPairs, Error, Event};
use frame_support::{assert_noop, assert_ok};
use frame_support::{
	sp_runtime::{
		traits::{Convert, EnsureAdd, EnsureDiv, EnsureMul, EnsureSub, IntegerSquareRoot, Zero},
		ArithmeticError, FixedU128, Perbill,
	},
	traits::{fungible, fungibles},
};
use humidefi_interface::DexHelpers;

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

		assert_ok!(Dex::new_liquidity(bob.clone(), asset_pairs, first_balance_a, first_balance_b));

		println!("");
		println!("Dex Account: {:?}", Dex::get_dex_account());
		println!("");

		println!("===============Create Liquidity===================");
		println!("Bob Liquidity Provide for Asset A: {:?}", FixedU128::from_inner(first_balance_a));
		println!("Bob Liquidity Provide for Asset B: {:?}", FixedU128::from_inner(first_balance_b));
		println!("Bob LP Token Balance: {:?}", FixedU128::from_inner(Dex::get_asset_balance(3, 2)));
		println!("Bob Asset A Balance: {:?}", FixedU128::from_inner(Dex::get_asset_balance(1, 2)));
		println!("Bob Asset B Balance: {:?}", FixedU128::from_inner(Dex::get_asset_balance(2, 2)));
		println!("===============Create Liquidity===================");
		println!("");
	});
}
