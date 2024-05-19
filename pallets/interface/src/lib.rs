#![cfg_attr(not(feature = "std"), no_std)]

use core::{fmt::Debug};
use codec::FullCodec;
use frame_support::{
	pallet_prelude::*,
	sp_runtime::{ BoundedVec,  traits::ConstU32, FixedU128 },
};

pub trait AssetPairsTrait {  type AssetPairs; }
pub trait LiquidityPoolTrait {  type LiquidityPool; }
pub trait AccountLiquidityPoolTrait {  type AccountLiquidityPool; }

pub trait DexCaller {
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + Ord + MaxEncodedLen;
	type AssetId: FullCodec + Clone + Eq + PartialEq + Debug + scale_info::TypeInfo + MaxEncodedLen;
	type AssetBalance: FullCodec + Copy + Default + Debug + scale_info::TypeInfo + MaxEncodedLen;
	type AccountLiquidityPoolId: Parameter + Member + MaybeSerializeDeserialize + Debug + Ord + MaxEncodedLen;
	type AssetPairs: AssetPairsTrait;

	fn new_liquidity(
		who: Self::AccountId,
		asset_pair: Self::AssetPairs,
		asset_x_balance: Self::AssetBalance,
		asset_y_balance: Self::AssetBalance
	) -> Result<(), DispatchError>;

	fn redeem_liquidity(
		who: Self::AccountId,
		asset_pair:  Self::AssetPairs,
		lp_token: Self::AssetId,
		id: Self::AccountLiquidityPoolId
	) -> Result<(), DispatchError>;

	fn swap_exact_in_for_out(
		who: Self::AccountId,
		asset_exact_in: Self::AssetId,
		asset_exact_in_balance: Self::AssetBalance,
		asset_max_out: Self::AssetId
	) -> Result<(), DispatchError>;

	fn swap_in_for_exact_out(
		who: Self::AccountId,
		asset_exact_out: Self::AssetId,
		asset_exact_out_balance: Self::AssetBalance,
		asset_min_in: Self::AssetId
	) -> Result<(), DispatchError>;

	fn transfer_asset(
		who: Self::AccountId,
		asset: Self::AssetId,
		asset_balance: Self::AssetBalance,
		account_id: Self::AccountId
	) -> Result<(), DispatchError>;
}

pub trait DexHelpers {
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + Ord + MaxEncodedLen;
	type AssetId: FullCodec + Clone + Eq + PartialEq + Debug + scale_info::TypeInfo + MaxEncodedLen;
	type AssetBalance: FullCodec + Copy + Default + Debug + scale_info::TypeInfo + MaxEncodedLen;
	type AccountLiquidityPoolId: Parameter + Member + MaybeSerializeDeserialize + Debug + Ord + MaxEncodedLen;
	type AssetPairs: AssetPairsTrait;
	type LiquidityPool: LiquidityPoolTrait;
	type AccountLiquidityPool: AccountLiquidityPoolTrait;

	fn get_dex_account() -> Self::AccountId;

	fn get_asset_balance(
		asset: Self::AssetId,
		account_id: Self::AccountId
	) -> Self::AssetBalance;

	fn get_liquidity_pool(
		asset_pair: Self::AssetPairs
	) -> Option<Self::LiquidityPool>;

	fn get_account_liquidity_pools(
		account_id: Self::AccountId,
		asset_pair: Self::AssetPairs
	) -> Option<BoundedVec<Self::AccountLiquidityPool, ConstU32<100>>>;

	fn check_asset_balance(
		account_id: Self::AccountId,
		asset: Self::AssetId,
		asset_balance: Self::AssetBalance
	) -> Result<(), DispatchError>;

	fn compute_and_mint_lp_token(
		asset_pair: Self::AssetPairs,
		asset_x_balance: Self::AssetBalance,
		asset_y_balance: Self::AssetBalance
	) -> Result<(Self::AssetId, Self::AssetBalance), DispatchError>;

	fn compute_price(
		asset_x_balance: Self::AssetBalance,
		asset_y_balance: Self::AssetBalance
	) -> Result<FixedU128, DispatchError>;

	fn compute_xy_assets(
		account_id: Self::AccountId,
		asset_pair: Self::AssetPairs,
		lp_token: Self::AssetId,
		id: Self::AccountLiquidityPoolId
	) -> Result<(Self::AssetBalance, Self::AssetBalance, Self::AssetBalance), DispatchError>;
}
