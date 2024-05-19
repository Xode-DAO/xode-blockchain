#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod humidefi;

extern crate humidefi_interface;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		sp_runtime::FixedU128,
		traits::{fungible, fungibles},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type NativeBalance: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::hold::Inspect<Self::AccountId>
			+ fungible::hold::Mutate<Self::AccountId>
			+ fungible::freeze::Inspect<Self::AccountId>
			+ fungible::freeze::Mutate<Self::AccountId>;

		type Fungibles: fungibles::Inspect<Self::AccountId, AssetId = u32, Balance = u128>
			+ fungibles::Mutate<Self::AccountId>
			+ fungibles::Create<Self::AccountId>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	pub type AssetIdOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::AssetId;

	pub type AssetBalanceOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	pub type AccountLiquidityPoolId = u64;

	#[derive(Clone, Eq, PartialEq, DebugNoBound, TypeInfo, Encode, Decode, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct AssetPairs<T: Config> {
		pub asset_x: AssetIdOf<T>,
		pub asset_y: AssetIdOf<T>,
	}

	#[derive(Clone, Eq, PartialEq, DebugNoBound, TypeInfo, Encode, Decode, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct LiquidityPool<T: Config> {
		pub asset_pair: AssetPairs<T>,
		pub asset_x_balance: FixedU128,
		pub asset_y_balance: FixedU128,
		pub price: FixedU128,
		pub asset_x_fee: FixedU128,
		pub asset_y_fee: FixedU128,
		pub lp_token: AssetIdOf<T>,
		pub lp_token_balance: FixedU128,
	}

	#[derive(Clone, Eq, PartialEq, DebugNoBound, TypeInfo, Encode, Decode, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct AccountLiquidityPool<T: Config> {
		pub id: AccountLiquidityPoolId,
		pub account_id: <T as frame_system::Config>::AccountId,
		pub asset_pair: AssetPairs<T>,
		pub asset_x_balance: FixedU128,
		pub asset_y_balance: FixedU128,
		pub lp_token: AssetIdOf<T>,
		pub lp_token_balance: FixedU128,
	}

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool_storage)]
	pub type LiquidityPoolStorage<T> = StorageMap<
		_,
		Blake2_128Concat,
		AssetPairs<T>,
		LiquidityPool<T>,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn account_liquidity_pool_storage)]
	pub type AccountLiquidityPoolStorage<T> = StorageMap<
		_,
		Blake2_128Concat,
		(<T as frame_system::Config>::AccountId, AssetPairs<T>),
		BoundedVec<AccountLiquidityPool<T>, ConstU32<100>>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		LiquidityAddedSuccessfully,
		LiquidityRedeemedSuccessfully,
		SwapExecutedSuccessfully,
		TransferExecutedSuccessfully
	}

	#[pallet::error]
	pub enum Error<T> {
		CheckAssetXBalanceError,
		CheckAssetYBalanceError,
		CheckAssetLiquidityPoolTokenBalanceError,
		CheckAssetSwapInBalanceError,
		CheckAssetSwapOutBalanceError,
		AssetDoesNotHaveEnoughBalance,

		ComputeAndMintLiquidityPoolTokenError,
		ComputePriceError,
		ComputeXYBalancesError,
		CannotBeZero,

		LiquidityPoolDoesNotExists,

		AccountLiquidityPoolBoundedVecError,
		AccountLiquidityPoolIdError,
		AccountLiquidityPoolDoesNotExists,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::default())]
		pub fn new_liquidity(
			origin: OriginFor<T>,
			asset_pair: AssetPairs<T>,
			asset_x_balance: AssetBalanceOf<T>,
			asset_y_balance: AssetBalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Pallet<T> as humidefi_interface::DexCaller>::new_liquidity(
				who,
				asset_pair,
				asset_x_balance,
				asset_y_balance
			)?;

			Self::deposit_event(Event::LiquidityAddedSuccessfully);
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::default())]
		pub fn redeem_liquidity(
			origin: OriginFor<T>,
			asset_pair: AssetPairs<T>,
			lp_token: AssetIdOf<T>,
			id: AccountLiquidityPoolId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Pallet<T> as humidefi_interface::DexCaller>::redeem_liquidity(
				who,
				asset_pair,
				lp_token,
				id
			)?;

			Self::deposit_event(Event::LiquidityRedeemedSuccessfully);
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::default())]
		pub fn swap_exact_in_for_out(
			origin: OriginFor<T>,
			asset_exact_in: AssetIdOf<T>,
			asset_exact_in_balance: AssetBalanceOf<T>,
			asset_max_out: AssetIdOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Pallet<T> as humidefi_interface::DexCaller>::swap_exact_in_for_out(
				who,
				asset_exact_in,
				asset_exact_in_balance,
				asset_max_out
			)?;

			Self::deposit_event(Event::SwapExecutedSuccessfully);
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::default())]
		pub fn swap_in_for_exact_out(
			origin: OriginFor<T>,
			asset_exact_out: AssetIdOf<T>,
			asset_exact_out_balance: AssetBalanceOf<T>,
			asset_min_in: AssetIdOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Pallet<T> as humidefi_interface::DexCaller>::swap_in_for_exact_out(
				who,
				asset_exact_out,
				asset_exact_out_balance,
				asset_min_in
			)?;

			Self::deposit_event(Event::SwapExecutedSuccessfully);
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::default())]
		pub fn transfer_asset(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			asset_balance: AssetBalanceOf<T>,
			account_id: <T as frame_system::Config>::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Pallet<T> as humidefi_interface::DexCaller>::transfer_asset(
				who,
				asset,
				asset_balance,
				account_id
			)?;

			Self::deposit_event(Event::TransferExecutedSuccessfully);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_dex_account() -> <T as frame_system::Config>::AccountId {
			<Pallet<T> as humidefi_interface::DexHelpers>::get_dex_account()
		}
	}
}
