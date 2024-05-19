use frame_support::{
	pallet_prelude::*,
	sp_runtime::{
        BoundedVec,
		traits::{
			AccountIdConversion,
			EnsureAdd, EnsureSub, EnsureDiv, EnsureMul,
			Zero,
			ConstU32
		},
		FixedU128, Perbill,
	},
	traits::{fungible, fungibles},
	PalletId,
};
use super::*;

use humidefi_interface::{
	AssetPairsTrait,
	LiquidityPoolTrait,
	AccountLiquidityPoolTrait
};
use humidefi_interface::DexCaller;
use humidefi_interface::DexHelpers;

const HUMIDEFI: PalletId = PalletId(*b"HUMIDEFI");

impl<T: Config> AssetPairsTrait for AssetPairs<T> { type AssetPairs = Self; }
impl<T: Config> LiquidityPoolTrait for LiquidityPool<T> { type LiquidityPool = Self; }
impl<T: Config> AccountLiquidityPoolTrait for AccountLiquidityPool<T> { type AccountLiquidityPool = Self; }

impl<T: Config> DexCaller for Pallet<T> {
	type AccountId = T::AccountId;
	type AssetId = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::AssetId;
	type AssetBalance = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::Balance;
	type AccountLiquidityPoolId = u64;
	type AssetPairs = <AssetPairs<T> as AssetPairsTrait>::AssetPairs;

	fn new_liquidity(
		who:  Self::AccountId,
		asset_pair: Self::AssetPairs,
		asset_x_balance:  Self::AssetBalance,
		asset_y_balance:  Self::AssetBalance,
	) -> Result<(), DispatchError> {
		let dex_account_id = <Pallet<T> as DexHelpers>::get_dex_account();

		<Pallet<T> as DexHelpers>::check_asset_balance(
			who.clone(),
			asset_pair.clone().asset_x,
			asset_x_balance,
		).expect(Error::<T>::CheckAssetXBalanceError.into());

		<Pallet<T> as DexHelpers>::check_asset_balance(
			who.clone(),
			asset_pair.clone().asset_y,
			asset_x_balance,
		).expect(Error::<T>::CheckAssetYBalanceError.into());

		<T::Fungibles as fungibles::Mutate<_>>::transfer(
			asset_pair.clone().asset_x,
			&who.clone(),
			&dex_account_id.clone(),
			asset_x_balance,
			frame_support::traits::tokens::Preservation::Expendable,
		)?;

		<T::Fungibles as fungibles::Mutate<_>>::transfer(
			asset_pair.clone().asset_y,
			&who.clone(),
			&dex_account_id.clone(),
			asset_y_balance,
			frame_support::traits::tokens::Preservation::Expendable,
		)?;

		let mint_liquidity = <Pallet<T> as DexHelpers>::compute_and_mint_lp_token(
			asset_pair.clone(),
			asset_x_balance,
			asset_y_balance,
		).expect(Error::<T>::ComputeAndMintLiquidityPoolTokenError.into());

		let lp_token: Self::AssetId = mint_liquidity.0;
		let lp_token_balance = mint_liquidity.1;

		<Pallet<T> as DexHelpers>::check_asset_balance(
			dex_account_id.clone(),
			lp_token,
			lp_token_balance,
		).expect(Error::<T>::CheckAssetLiquidityPoolTokenBalanceError.into());

		<T::Fungibles as fungibles::Mutate<_>>::transfer(
			lp_token.clone(),
			&dex_account_id.clone(),
			&who.clone(),
			lp_token_balance,
			frame_support::traits::tokens::Preservation::Expendable,
		)?;

		let get_liquidity_pool = <Pallet<T> as DexHelpers>::get_liquidity_pool(asset_pair.clone());
		match get_liquidity_pool {
			Some(liquidity_pool) => {
				let update_asset_x_balance = liquidity_pool
					.asset_x_balance
					.add(FixedU128::from_inner(asset_x_balance));

				let update_asset_y_balance = liquidity_pool
					.asset_y_balance
					.add(FixedU128::from_inner(asset_y_balance));

				let update_price = <Pallet<T> as DexHelpers>::compute_price(
					update_asset_x_balance.into_inner(),
					update_asset_y_balance.into_inner()
				).expect(Error::<T>::ComputePriceError.into());

				let update_lp_token_balance = liquidity_pool
					.lp_token_balance
					.add(FixedU128::from_inner(lp_token_balance));

				LiquidityPoolStorage::<T>::mutate(asset_pair.clone(), |query| {
					let liquidity_pool_payload = LiquidityPool::<T> {
						asset_pair: asset_pair.clone(),
						asset_x_balance: update_asset_x_balance,
						asset_y_balance: update_asset_y_balance,
						price: update_price,
						asset_x_fee: liquidity_pool.asset_x_fee,
						asset_y_fee: liquidity_pool.asset_y_fee,
						lp_token: liquidity_pool.lp_token,
						lp_token_balance: update_lp_token_balance,
					};

					*query = Some(liquidity_pool_payload);
				});
			},
			None => {
				let new_price = <Pallet<T> as DexHelpers>::compute_price(
					asset_x_balance,
					asset_y_balance
				).expect(Error::<T>::ComputePriceError.into());

				let liquidity_pool_payload = LiquidityPool::<T> {
					asset_pair: asset_pair.clone(),
					asset_x_balance: FixedU128::from_inner(asset_x_balance),
					asset_y_balance: FixedU128::from_inner(asset_y_balance),
					price: new_price,
					asset_x_fee: FixedU128::from_inner(0),
					asset_y_fee: FixedU128::from_inner(0),
					lp_token,
					lp_token_balance: FixedU128::from_inner(lp_token_balance),
				};

				LiquidityPoolStorage::<T>::insert(asset_pair.clone(), liquidity_pool_payload);
			}
		}

		let mut account_liquidity_pool_payload = AccountLiquidityPool::<T> {
			id: 1u64.into(),
			account_id: who.clone(),
			asset_pair: asset_pair.clone(),
			asset_x_balance: FixedU128::from_inner(asset_x_balance),
			asset_y_balance: FixedU128::from_inner(asset_y_balance),
			lp_token,
			lp_token_balance: FixedU128::from_inner(lp_token_balance),
		};

		let get_account_liquidity_pools = <Pallet<T> as DexHelpers>::get_account_liquidity_pools(who.clone(), asset_pair.clone());
		match get_account_liquidity_pools {
			Some(account_liquidity_pools) => {
				let mut last_id = 0u64.into();
				if let Some(account_liquidity_pool) = account_liquidity_pools.last() {
					last_id = account_liquidity_pool.id;
				}

				account_liquidity_pool_payload.id = last_id
					.ensure_add(1)
					.expect(Error::<T>::AccountLiquidityPoolIdError.into());

				let mut mutate_account_liquidity_pools = account_liquidity_pools.clone();
				mutate_account_liquidity_pools
					.try_push(account_liquidity_pool_payload.clone())
					.expect(Error::<T>::AccountLiquidityPoolBoundedVecError.into());

				let storage_key = (who.clone(), asset_pair.clone());
				AccountLiquidityPoolStorage::<T>::mutate(storage_key, |mut query| {
					let update_account_liquidity_pools = mutate_account_liquidity_pools.clone();
					*query = Some(update_account_liquidity_pools)
				});
			},
			None => {
				let mut new_account_liquidity_pools: BoundedVec<
					AccountLiquidityPool<T>,
					ConstU32<100>,
				> = Default::default();

				new_account_liquidity_pools
					.try_push(account_liquidity_pool_payload.clone())
					.expect(Error::<T>::AccountLiquidityPoolBoundedVecError.into());

				AccountLiquidityPoolStorage::<T>::insert(
					(who, asset_pair),
					new_account_liquidity_pools,
				);
			}
		}

		Ok(())
	}

	fn redeem_liquidity(
		who: Self::AccountId,
		asset_pair: Self::AssetPairs,
		lp_token: Self::AssetId,
		id: Self::AccountLiquidityPoolId,
	) -> Result<(), DispatchError> {
		let get_liquidity_pool = <Pallet<T> as DexHelpers>::get_liquidity_pool(asset_pair.clone());
		if !get_liquidity_pool.is_some() {
			return Err(Error::<T>::LiquidityPoolDoesNotExists.into())
		}

		let asset_xy_balances = <Pallet<T> as DexHelpers>::compute_xy_assets(
			who.clone(),
			asset_pair.clone(),
			lp_token,
			id
		).expect(Error::<T>::ComputeXYBalancesError.into());

		let asset_x_balance = asset_xy_balances.0;
		let asset_y_balance = asset_xy_balances.1;
		let lp_token_balance = asset_xy_balances.2;

		let dex_account_id = <Pallet<T> as DexHelpers>::get_dex_account();

		<Pallet<T> as DexHelpers>::check_asset_balance(
			dex_account_id.clone(),
			asset_pair.clone().asset_x,
			asset_x_balance,
		).expect(Error::<T>::CheckAssetXBalanceError.into());

		<Pallet<T> as DexHelpers>::check_asset_balance(
			dex_account_id.clone(),
			asset_pair.clone().asset_x,
			asset_y_balance,
		).expect(Error::<T>::CheckAssetYBalanceError.into());

		<T::Fungibles as fungibles::Mutate<_>>::transfer(
			asset_pair.clone().asset_x,
			&dex_account_id.clone(),
			&who.clone(),
			asset_x_balance,
			frame_support::traits::tokens::Preservation::Expendable,
		)?;

		<T::Fungibles as fungibles::Mutate<_>>::transfer(
			asset_pair.clone().asset_y,
			&dex_account_id.clone(),
			&who.clone(),
			asset_y_balance,
			frame_support::traits::tokens::Preservation::Expendable,
		)?;

		LiquidityPoolStorage::<T>::mutate(asset_pair.clone(), |mut query| {
			if let Some(mutate_liquidity_pool) = query {
				let update_asset_x_balance = mutate_liquidity_pool
					.asset_x_balance
					.sub(FixedU128::from_inner(asset_x_balance));

				let update_asset_y_balance = mutate_liquidity_pool
					.asset_y_balance
					.sub(FixedU128::from_inner(asset_y_balance));

				let update_price = <Pallet<T> as DexHelpers>::compute_price(
					update_asset_x_balance.into_inner(),
					update_asset_y_balance.into_inner()
				).expect(Error::<T>::ComputePriceError.into());

				let update_lp_token_balance = mutate_liquidity_pool
					.lp_token_balance
					.sub(FixedU128::from_inner(lp_token_balance));

				let liquidity_pool_payload = LiquidityPool::<T> {
					asset_pair: asset_pair.clone(),
					asset_x_balance: update_asset_x_balance,
					asset_y_balance: update_asset_y_balance,
					price: update_price,
					asset_x_fee: mutate_liquidity_pool.asset_x_fee,
					asset_y_fee: mutate_liquidity_pool.asset_y_fee,
					lp_token: mutate_liquidity_pool.lp_token,
					lp_token_balance: update_lp_token_balance,
				};

				*query = Some(liquidity_pool_payload);
			}
		});

		AccountLiquidityPoolStorage::<T>::remove((who.clone(), asset_pair.clone()));

		Ok(())
	}

	fn swap_exact_in_for_out(
		who: Self::AccountId,
		asset_exact_in: Self::AssetId,
		asset_exact_in_balance: Self::AssetBalance,
		asset_max_out: Self::AssetId,
	) -> Result<(), DispatchError> {
		let dex_account_id = <Pallet<T> as DexHelpers>::get_dex_account();

		<Pallet<T> as DexHelpers>::check_asset_balance(
			who.clone(),
			asset_exact_in,
			asset_exact_in_balance,
		).expect(Error::<T>::CheckAssetSwapInBalanceError.into());

		<T::Fungibles as fungibles::Mutate<_>>::transfer(
			asset_exact_in,
			&who.clone(),
			&dex_account_id.clone(),
			asset_exact_in_balance,
			frame_support::traits::tokens::Preservation::Expendable,
		)?;

		let asset_pair = AssetPairs::<T> { asset_x: asset_exact_in, asset_y: asset_max_out };
		let get_liquidity_pool = <Pallet<T> as DexHelpers>::get_liquidity_pool(asset_pair.clone());
		match get_liquidity_pool {
			Some(liquidity_pool) => {
				let mut price = FixedU128::from_inner(0);

				if asset_exact_in == liquidity_pool.asset_pair.asset_x {
					price = <Pallet<T> as DexHelpers>::compute_price(
						liquidity_pool.asset_x_balance.into_inner(),
						liquidity_pool.asset_y_balance.into_inner()
					).expect(Error::<T>::ComputePriceError.into());
				}

				if asset_exact_in == liquidity_pool.asset_pair.asset_y {
					price = <Pallet<T> as DexHelpers>::compute_price(
						liquidity_pool.asset_y_balance.into_inner(),
						liquidity_pool.asset_x_balance.into_inner()
					).expect(Error::<T>::ComputePriceError.into());
				}

				let asset_max_out_balance = FixedU128::from_inner(price.into_inner()).mul(FixedU128::from_inner(asset_exact_in_balance));

				<Pallet<T> as DexHelpers>::check_asset_balance(
					dex_account_id.clone(),
					asset_max_out,
					asset_max_out_balance.into_inner(),
				).expect(Error::<T>::CheckAssetSwapOutBalanceError.into());

				<T::Fungibles as fungibles::Mutate<_>>::transfer(
					asset_max_out,
					&dex_account_id.clone(),
					&who.clone(),
					asset_max_out_balance.into_inner(),
					frame_support::traits::tokens::Preservation::Expendable,
				)?;

				let mut update_asset_x_balance = FixedU128::from_inner(0);
				let mut update_asset_y_balance = FixedU128::from_inner(0);

				if asset_exact_in == liquidity_pool.asset_pair.asset_x {
					update_asset_x_balance = liquidity_pool
						.asset_x_balance
						.add(FixedU128::from_inner(asset_exact_in_balance));

					update_asset_y_balance = liquidity_pool
						.asset_y_balance
						.sub(asset_max_out_balance);
				}

				if asset_exact_in == liquidity_pool.asset_pair.asset_y {
					update_asset_x_balance = liquidity_pool
						.asset_x_balance
						.sub(asset_max_out_balance);

					update_asset_y_balance = liquidity_pool
						.asset_y_balance
						.add(FixedU128::from_inner(asset_exact_in_balance));
				}

				let update_price = <Pallet<T> as DexHelpers>::compute_price(
					update_asset_x_balance.into_inner(),
					update_asset_y_balance.into_inner()
				).expect(Error::<T>::ComputePriceError.into());

				LiquidityPoolStorage::<T>::mutate(asset_pair.clone(), |query| {
					let liquidity_pool_payload = LiquidityPool::<T> {
						asset_pair: asset_pair.clone(),
						asset_x_balance: update_asset_x_balance,
						asset_y_balance: update_asset_y_balance,
						price: update_price,
						asset_x_fee: liquidity_pool.asset_x_fee,
						asset_y_fee: liquidity_pool.asset_y_fee,
						lp_token: liquidity_pool.lp_token,
						lp_token_balance: liquidity_pool.lp_token_balance,
					};

					*query = Some(liquidity_pool_payload);
				});
			},
			None => {
				return Err(Error::<T>::LiquidityPoolDoesNotExists.into())
			}
		}

		Ok(())
	}

	fn swap_in_for_exact_out(
		who: Self::AccountId,
		asset_exact_out: Self::AssetId,
		asset_exact_out_balance: Self::AssetBalance,
		asset_min_in: Self::AssetId,
	) -> Result<(), DispatchError> {
		let dex_account_id = <Pallet<T> as DexHelpers>::get_dex_account();

		<Pallet<T> as DexHelpers>::check_asset_balance(
			dex_account_id.clone(),
			asset_exact_out,
			asset_exact_out_balance,
		).expect(Error::<T>::CheckAssetSwapOutBalanceError.into());

		<T::Fungibles as fungibles::Mutate<_>>::transfer(
			asset_exact_out,
			&dex_account_id.clone(),
			&who.clone(),
			asset_exact_out_balance,
			frame_support::traits::tokens::Preservation::Expendable,
		)?;

		let asset_pair = AssetPairs::<T> { asset_x: asset_min_in, asset_y: asset_exact_out };
		let get_liquidity_pool = <Pallet<T> as DexHelpers>::get_liquidity_pool(asset_pair.clone());
		match get_liquidity_pool {
			Some(liquidity_pool) => {
				let mut price = FixedU128::from_inner(0);

				if asset_min_in == liquidity_pool.asset_pair.asset_x {
					price = <Pallet<T> as DexHelpers>::compute_price(
						liquidity_pool.asset_x_balance.into_inner(),
						liquidity_pool.asset_y_balance.into_inner()
					).expect(Error::<T>::ComputePriceError.into());
				}

				if asset_min_in == liquidity_pool.asset_pair.asset_y {
					price = <Pallet<T> as DexHelpers>::compute_price(
						liquidity_pool.asset_y_balance.into_inner(),
						liquidity_pool.asset_x_balance.into_inner()
					).expect(Error::<T>::ComputePriceError.into());
				}

				let asset_min_in_balance = FixedU128::from_inner(price.into_inner()).mul(FixedU128::from_inner(asset_exact_out_balance));

				<Pallet<T> as DexHelpers>::check_asset_balance(
					who.clone(),
					asset_min_in,
					asset_min_in_balance.into_inner(),
				).expect(Error::<T>::CheckAssetSwapInBalanceError.into());

				<T::Fungibles as fungibles::Mutate<_>>::transfer(
					asset_min_in,
					&who.clone(),
					&dex_account_id.clone(),
					asset_min_in_balance.into_inner(),
					frame_support::traits::tokens::Preservation::Expendable,
				)?;

				let mut update_asset_x_balance = FixedU128::from_inner(0);
				let mut update_asset_y_balance = FixedU128::from_inner(0);

				if asset_min_in == liquidity_pool.asset_pair.asset_x {
					update_asset_x_balance = liquidity_pool
						.asset_x_balance
						.add(asset_min_in_balance);

					update_asset_y_balance = liquidity_pool
						.asset_y_balance
						.sub(FixedU128::from_inner(asset_exact_out_balance));
				}

				if asset_min_in == liquidity_pool.asset_pair.asset_y {
					update_asset_x_balance = liquidity_pool
						.asset_x_balance
						.sub(FixedU128::from_inner(asset_exact_out_balance));

					update_asset_y_balance = liquidity_pool
						.asset_y_balance
						.add(asset_min_in_balance);
				}

				let update_price = <Pallet<T> as DexHelpers>::compute_price(
					update_asset_x_balance.into_inner(),
					update_asset_y_balance.into_inner()
				).expect(Error::<T>::ComputePriceError.into());

				LiquidityPoolStorage::<T>::mutate(asset_pair.clone(), |query| {
					let liquidity_pool_payload = LiquidityPool::<T> {
						asset_pair: asset_pair.clone(),
						asset_x_balance: update_asset_x_balance,
						asset_y_balance: update_asset_y_balance,
						price: update_price,
						asset_x_fee: liquidity_pool.asset_x_fee,
						asset_y_fee: liquidity_pool.asset_y_fee,
						lp_token: liquidity_pool.lp_token,
						lp_token_balance: liquidity_pool.lp_token_balance,
					};

					*query = Some(liquidity_pool_payload);
				});
			},
			None => {
				return Err(Error::<T>::LiquidityPoolDoesNotExists.into())
			}
		}

		Ok(())
	}

	fn transfer_asset(
		who: Self::AccountId,
		asset: Self::AssetId,
		asset_balance: Self::AssetBalance,
		account_id: Self::AccountId,
	) -> Result<(), DispatchError> {
		<Pallet<T> as DexHelpers>::check_asset_balance(
			who.clone(),
			asset,
			asset_balance,
		).expect(Error::<T>::CheckAssetSwapInBalanceError.into());

		<T::Fungibles as fungibles::Mutate<_>>::transfer(
			asset,
			&who.clone(),
			&account_id.clone(),
			asset_balance,
			frame_support::traits::tokens::Preservation::Expendable,
		)?;

		Ok(())
	}
}

impl<T: Config> DexHelpers for Pallet<T> {
	type AccountId = T::AccountId;
	type AssetId = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::AssetId;
	type AssetBalance = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::Balance;
	type AccountLiquidityPoolId = u64;
	type AssetPairs = <AssetPairs<T> as AssetPairsTrait>::AssetPairs;
	type LiquidityPool = <LiquidityPool<T> as LiquidityPoolTrait>::LiquidityPool;
	type AccountLiquidityPool = <AccountLiquidityPool<T> as AccountLiquidityPoolTrait>::AccountLiquidityPool;

	fn get_dex_account() -> Self::AccountId {
		HUMIDEFI.into_account_truncating()
	}

	fn get_asset_balance(
		asset: Self::AssetId,
		account_id: Self::AccountId,
	) -> Self::AssetBalance {
		let balance = <T::Fungibles as fungibles::Inspect<_>>::balance(asset, &account_id);
		balance
	}

	fn get_liquidity_pool(
		asset_pair: Self::AssetPairs
	) -> Option<LiquidityPool<T>> {
		let existing_liquidity_pool = LiquidityPoolStorage::<T>::get(asset_pair.clone());
		match existing_liquidity_pool {
			Some(liquidity_pool) => return Some(liquidity_pool),
			None => {
				let swap_asset_pair = AssetPairs::<T> {
					asset_x: asset_pair.clone().asset_y,
					asset_y: asset_pair.clone().asset_x,
				};

				let liquidity_pool_swap_pair = LiquidityPoolStorage::<T>::get(swap_asset_pair);
				if let Some(liquidity_pool) = liquidity_pool_swap_pair {
					return Some(liquidity_pool)
				}

				return None
			},
		}
	}

	fn get_account_liquidity_pools(
		account_id: Self::AccountId,
		asset_pair: Self::AssetPairs,
	) -> Option<BoundedVec<AccountLiquidityPool<T>, ConstU32<100>>> {
		let storage_key = (account_id.clone(), asset_pair.clone());
		let existing_account_liquidity_pools = AccountLiquidityPoolStorage::<T>::get(storage_key);
		match existing_account_liquidity_pools {
			Some(account_liquidity_pools) => return Some(account_liquidity_pools),
			None => {
				let swap_asset_pair = AssetPairs::<T> {
					asset_x: asset_pair.clone().asset_y,
					asset_y: asset_pair.clone().asset_x,
				};

				let storage_key_swap_pair = (account_id.clone(), swap_asset_pair.clone());
				let account_liquidity_pools_swap_pair =
					AccountLiquidityPoolStorage::<T>::get(storage_key_swap_pair);

				if let Some(account_liquidity_pool) = account_liquidity_pools_swap_pair {
					return Some(account_liquidity_pool)
				}

				return None
			},
		}
	}

	fn check_asset_balance(
		account_id: Self::AccountId,
		asset: Self::AssetId,
		asset_balance: Self::AssetBalance,
	) -> Result<(), DispatchError> {
		let current_asset_balance = Self::get_asset_balance(asset, account_id.clone());

		current_asset_balance
			.ensure_sub(asset_balance)
			.expect(Error::<T>::AssetDoesNotHaveEnoughBalance.into());

		Ok(())
	}

	fn compute_and_mint_lp_token(
		asset_pair: Self::AssetPairs,
		asset_x_balance: Self::AssetBalance,
		asset_y_balance: Self::AssetBalance,
	) -> Result<(AssetIdOf<T>, AssetBalanceOf<T>), DispatchError> {
		let mut lp_token: AssetIdOf<T> = 1u32.into();
		let dex_account_id = Self::get_dex_account();

		let existing_liquidity_pool = Self::get_liquidity_pool(asset_pair.clone());

		match existing_liquidity_pool {
			Some(liquidity_pool) => {
				lp_token = liquidity_pool.lp_token;
			},
			None => {
				let mut counter = 1u32.into();

				loop {
					lp_token = counter;

					if !<T::Fungibles as fungibles::Inspect<_>>::asset_exists(lp_token) {
						<T::Fungibles as fungibles::Create<_>>::create(
							lp_token,
							dex_account_id.clone(),
							true,
							1u128.into(),
						)?;

						break;
					}

					counter += 1;
				}
			},
		}

		let mul_xy_assets = FixedU128::from_inner(asset_x_balance).mul(FixedU128::from_inner(asset_y_balance));
		if mul_xy_assets.is_zero() {
			return Err(Error::<T>::CannotBeZero.into())
		}

		let lp_token_balance = mul_xy_assets.sqrt().into_inner();

		<T::Fungibles as fungibles::Mutate<_>>::mint_into(
			lp_token,
			&dex_account_id.clone(),
			lp_token_balance,
		)?;

		Ok((lp_token, lp_token_balance))
	}

	fn compute_price(
		asset_x_balance: Self::AssetBalance,
		asset_y_balance: Self::AssetBalance,
	) -> Result<FixedU128, DispatchError> {
		if FixedU128::from_inner(asset_x_balance).is_zero() || FixedU128::from_inner(asset_y_balance).is_zero() {
			return Err(Error::<T>::CannotBeZero.into())
		}

		let price = FixedU128::from_rational(asset_y_balance, asset_x_balance);
		Ok(price)
	}

	fn compute_xy_assets(
		account_id: Self::AccountId,
		asset_pair: Self::AssetPairs,
		lp_token: Self::AssetId,
		id: Self::AccountLiquidityPoolId,
	) -> Result<(AssetBalanceOf<T>, AssetBalanceOf<T>, AssetBalanceOf<T>), DispatchError> {
		let existing_account_liquidity_pools = Self::get_account_liquidity_pools(account_id.clone(), asset_pair.clone());
		if !existing_account_liquidity_pools.is_some() {
			return Err(Error::<T>::AccountLiquidityPoolDoesNotExists.into())
		}

		let mut lp_token_balance = FixedU128::from_inner(0);
		if let Some(account_liquidity_pools) = existing_account_liquidity_pools {
			if account_liquidity_pools.to_vec().len() > 0 {
				for account_liquidity_pool in account_liquidity_pools {
					if account_liquidity_pool.lp_token == lp_token && account_liquidity_pool.id == id {
						lp_token_balance = account_liquidity_pool.lp_token_balance;
						break;
					}
				}
			}
		};

		let mut price = FixedU128::from_inner(0);
		let existing_liquidity_pool = LiquidityPoolStorage::<T>::get(asset_pair.clone());
		if let Some(liquidity_pool) = existing_liquidity_pool {
			price = liquidity_pool.price;
		}

		if price.is_zero() {
			return Err(Error::<T>::CannotBeZero.into())
		}

		let get_asset_x_balance = lp_token_balance.div(FixedU128::from_inner(price.into_inner()).sqrt());
		let get_asset_y_balance = lp_token_balance.mul(FixedU128::from_inner(price.into_inner()).sqrt());
		let get_lp_token_balance = lp_token_balance;

		Ok((get_asset_x_balance.into_inner(), get_asset_y_balance.into_inner(), get_lp_token_balance.into_inner()))
	}
}
