//! DEX Pallet
//!
//! This pallet provides functionality for an constant product market maker
//! Similar to Uniswap v2.
//!
//! # Overview:
//! A decentralized automated market maker allows users to buy and sell
//! assets where the counterparty is the liqudity pool itself
//! rather than another user (unlike orderbook based exchanges),
//! This has the advantage of "always on" liquidity.
//! This is achieved by keeping the value of the liquidity pool constant,
//! so only the prices change as the quantity changes
//!
//! # Interface:
//! create_market_pool: Allows the user to create a liquidity pool with some initial balance
//! deposit_liquidity: Allows the user to add liqudity to a pool to earn part of the collected fees
//! withdraw_liquidity: Allows the user to remove his liquidity from a pool
//! buy: Allows the user to exchange the QUOTE asset for the BASE asset
//! sell: Allows the user to exchange the BASE asset for the QUOTE asset
//!
//! # Hooks:
//! The offchain worker calls a function every 10 blocks
//! which perform the payout to the liquidity providers as a reward

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

use frame_support::{
	inherent::Vec,
	traits::{
		tokens::fungibles::{Inspect, Transfer},
		Get,
	},
	transactional, PalletId,
};
pub use pallet::*;
use sp_runtime::{traits::Zero, DispatchError};

use sp_runtime::traits::AccountIdConversion;
use types::*;

mod types;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The ubiqutous event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The taker fee a user pays for taking liquidity and doing the asset swap
		/// First item is the numerator, second one the denominator
		/// fee_rate = numerator / denominator.
		#[pallet::constant]
		type TakerFee: Get<(u32, u32)>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The type that enables currency transfers
		type Currencies: Transfer<Self::AccountId, Balance = u128, AssetId = u8>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Stores information about the markets liquidity pool
	///
	/// Maps Market => (BASE Balance, QUOTE Balance)
	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool)]
	pub type LiquidityPool<T: Config> =
		StorageMap<_, Blake2_128Concat, Market<T>, MarketInfo<T>, OptionQuery>;

	/// Stores information regarding the liquidity provision of users in a given market
	/// Used for rewarding liquidity providers from the collected taker fees.
	///
	/// Maps Market and Account => (BASE Balance, QUOTE Balance)
	#[pallet::storage]
	#[pallet::getter(fn liq_provision_pool)]
	pub type LiqProvisionPool<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Market<T>,
		Blake2_128Concat,
		T::AccountId,
		(BalanceOf<T>, BalanceOf<T>),
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A liquidity pool has been created for a trading pair
		///
		/// # Fields:
		/// 0: Who created the market
		/// 1: The market identifier
		/// 2: Liquidity for BASE asset
		/// 3: Liquidity for QUOTE asset
		PoolCreated(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),

		/// Emitted when liquidity has been added to a pool
		///
		/// # Fields:
		/// 0: The liquidity provider account
		/// 1: The market identifier for which liquidity has been added
		/// 2: The BASE asset balance added
		/// 3: The QUOT asset balance added
		LiquidityAdded(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),

		/// Emitted when a user removes liquidity from a pool
		///
		/// # Fields:
		/// 0: The account withdrawing the liquidity
		/// 1: The market it's been withdrawn from
		/// 2: The amount of BASE asset withdrawn
		/// 3: The amount of QUOTE asset withdrawn
		LiquidityWithdrawn(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),

		/// A user bought the BASE asset
		///
		/// # Fields:
		/// 0: The account which bought
		/// 1: The market in which it was bough
		/// 2: The amount of QUOTE asset that was spent
		/// 3: The amount of BASE asset received
		Bought(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),

		/// A user sold the BASE asset
		///
		/// # Fields:
		/// 0: The account which sold
		/// 1: The market in which it was sold
		/// 2: The amount of BASE asset that was sold
		/// 3: The amount of QUOTE asset received
		Sold(T::AccountId, Market<T>, BalanceOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The market already exists and cannot be created
		MarketExists,

		/// The market the user specified does not exist
		MarketDoesNotExist,

		/// The user does not have enough balance
		NotEnoughBalance,

		/// Some arithmetic error occurred
		Arithmetic,

		/// originates from T::Currencies::transfer basically
		Transfer,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(now: BlockNumberFor<T>) {
			// Reward the liquidity providers every 10 blocks
			if now % 10u32.into() == Zero::zero() {
				if let Err(e) = Self::do_liquidity_provider_payout() {
					log::error!("do_liquidity_provider_payout failed due to {:?}", e);
				}
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a new pool for a market if it does not exist already
		/// The user is required to provide both BASE and QUOTE asset
		/// to bootstrap the liquidity of the pool
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// base_asset: The BASE asset of the market
		/// quote_asset: The QUOTE asset of the market
		/// base_amount: Amount of BASE currency to use for bootstrapping liquidity
		/// quote_amount: Amount of QUOTE currency to use for bootstrapping liquidity
		///
		/// # Weight:
		/// Requires base weight + 3 reads and 2 writes
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3, 2))]
		#[transactional] // This Dispatchable is atomic
		pub fn create_market_pool(
			origin: OriginFor<T>,
			base_asset: AssetIdOf<T>,
			quote_asset: AssetIdOf<T>,
			base_amount: BalanceOf<T>,
			quote_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// check if market pool exists already
			let market = (base_asset, quote_asset);
			ensure!(LiquidityPool::<T>::get(market).is_none(), Error::<T>::MarketExists);

			// Check that balance of BASE asset of caller account is sufficient
			let base_balance = Self::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			// Check if balance of QUOTE asset of caller account is sufficient
			let quote_balance = Self::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			let pool_account = Self::pool_account();

			// Transfer the BASE currency into the pool
			<T as Config>::Currencies::transfer(
				base_asset,
				&who,
				&pool_account,
				base_amount,
				true,
			)?;
			// Transfer the QUOTE currency into the pool
			<T as Config>::Currencies::transfer(
				quote_asset,
				&who,
				&pool_account,
				quote_amount,
				true,
			)?;

			// Insert the balance information for the market
			let market_info = MarketInfo {
				base_balance: base_amount,
				quote_balance: quote_amount,
				collected_base_fees: Zero::zero(),
				collected_quote_fees: Zero::zero(),
			};
			LiquidityPool::<T>::insert(market, market_info);

			// remember who depsited what in the liquidity provision pool
			LiqProvisionPool::<T>::insert(market, who.clone(), (base_amount, quote_amount));

			// Emit the event that the pool has been created
			Self::deposit_event(Event::PoolCreated(who, market, base_amount, quote_amount));

			Ok(())
		}

		/// Allows the user to deposit liquidity to a pool,
		/// allowing for rewards to be generated on the deposit.
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// market: To which market the liquidity should be added
		/// base_amount: The amount of BASE asset to deposit
		/// quote_amount: The amount of QUOTE asset to deposit
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		#[transactional] // This Dispatchable is atomic
		pub fn deposit_liquidity(
			origin: OriginFor<T>,
			market: Market<T>,
			base_amount: BalanceOf<T>,
			quote_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let (base_asset, quote_asset) = market;

			// check if market pool exists
			ensure!(LiquidityPool::<T>::get(market).is_some(), Error::<T>::MarketDoesNotExist);

			// Check that balance of BASE asset of caller account is sufficient
			let base_balance = Self::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			// Check if balance of QUOTE asset of caller account is sufficient
			let quote_balance = Self::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// Use try_mutate in case the closure fails, e.g.: arithmetic overflow
			LiquidityPool::<T>::try_mutate(market, |opt_market_info| -> DispatchResult {
				let market_info = opt_market_info
					.clone()
					.expect("Check that the market pool exists has been done before; qed");

				market_info
					.base_balance
					.checked_add(base_amount)
					.ok_or(Error::<T>::Arithmetic)?;
				market_info
					.quote_balance
					.checked_add(quote_amount)
					.ok_or(Error::<T>::Arithmetic)?;

				Ok(())
			})?;

			let pool_account = Self::pool_account();

			// transfer the BASE currency to pool account
			<T as Config>::Currencies::transfer(
				base_asset,
				&who,
				&pool_account,
				base_amount,
				true,
			)?;
			// transfer the QUOTE currency to pool account
			<T as Config>::Currencies::transfer(
				quote_asset,
				&who,
				&pool_account,
				quote_amount,
				true,
			)?;

			// Keep track of liquidity providers
			LiqProvisionPool::<T>::try_mutate(
				market,
				who.clone(),
				|(base_balance, quote_balance)| -> DispatchResult {
					*base_balance =
						base_balance.checked_add(base_amount).ok_or(Error::<T>::Arithmetic)?;
					*quote_balance =
						quote_balance.checked_add(quote_amount).ok_or(Error::<T>::Arithmetic)?;

					Ok(())
				},
			)?;

			Self::deposit_event(Event::LiquidityAdded(who, market, base_amount, quote_amount));

			Ok(())
		}

		/// Allows the user to withdraw his liquidity from a pool
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// market: The liquidity pool to withdraw from
		/// base_amount: The amount of the BASE asset to withdraw
		/// quote_amount: The amount of the QUOTE asset to withdraw
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		#[transactional] // This Dispatchable is atomic
		pub fn withdraw_liquidity(
			origin: OriginFor<T>,
			market: Market<T>,
			base_amount: BalanceOf<T>,
			quote_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check that the market exists
			ensure!(LiquidityPool::<T>::get(market).is_some(), Error::<T>::MarketDoesNotExist);

			let (base_asset, quote_asset) = market;
			let pool_account = Self::pool_account();

			// ensure the user has enough balance in the pool to withdraw
			let (users_base_balance, users_quote_balance) =
				LiqProvisionPool::<T>::get(market, &who);
			ensure!(users_base_balance >= base_amount, Error::<T>::NotEnoughBalance);
			ensure!(users_quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// transfer out BASE asset from pool
			<T as Config>::Currencies::transfer(
				base_asset,
				&pool_account,
				&who,
				base_amount,
				true,
			)?;
			// transfer out QUOTE asset from pool
			<T as Config>::Currencies::transfer(
				quote_asset,
				&pool_account,
				&who,
				quote_amount,
				true,
			)?;

			// update LiqProvisionPool
			LiqProvisionPool::<T>::try_mutate(
				market,
				who.clone(),
				|(base_balance, quote_balance)| -> DispatchResult {
					*base_balance =
						base_balance.checked_sub(base_amount).ok_or(Error::<T>::Arithmetic)?;
					*quote_balance =
						quote_balance.checked_sub(quote_amount).ok_or(Error::<T>::Arithmetic)?;

					Ok(())
				},
			)?;

			Self::deposit_event(Event::LiquidityWithdrawn(who, market, base_amount, quote_amount));

			Ok(())
		}

		/// Allows the user to buy the BASE asset of a market
		///
		/// # Arguments
		/// origin: The obiquitous origin of a transaction
		/// market: The market in which the user wants to trade
		/// quote_amount: The amount of the QUOTE asset the user is willing to spend
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		#[transactional] // This Dispatchable is atomic
		pub fn buy(
			origin: OriginFor<T>,
			market: Market<T>,
			quote_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// get balance of pool, if it exists
			let market_info =
				LiquidityPool::<T>::get(market).ok_or(Error::<T>::MarketDoesNotExist)?;

			let (base_asset, quote_asset) = market;

			// Check that balance of QUOTE asset of caller account is sufficient
			let quote_balance = Self::balance(quote_asset, &who);
			ensure!(quote_balance >= quote_amount, Error::<T>::NotEnoughBalance);

			// get the amount to receive
			let receive_amount = Self::get_received_amount(
				market_info.base_balance,
				market_info.quote_balance,
				OrderType::Buy,
				quote_amount,
			)?;
			let fee_quote = Self::fee_from_amount(quote_amount)?;
			// This is the amount of QUOTE currency being deposited into the pool
			let deposit_amount =
				quote_amount.checked_sub(fee_quote).ok_or(Error::<T>::Arithmetic)?;

			let pool_account = Self::pool_account();

			// Transfer the QUOTE asset into the pool
			<T as Config>::Currencies::transfer(
				quote_asset,
				&who,
				&pool_account,
				deposit_amount,
				true,
			)?;
			// And get the BASE asset out of the pool
			<T as Config>::Currencies::transfer(
				base_asset,
				&pool_account,
				&who,
				receive_amount,
				true,
			)?;

			// Transfer the taker fee to a separate account
			let pool_fee_account = Self::pool_fee_account();
			<T as Config>::Currencies::transfer(
				quote_asset,
				&who,
				&pool_fee_account,
				fee_quote,
				true,
			)?;

			// update the market_info collected
			let fee_quote = Self::fee_from_amount(quote_amount)?;
			LiquidityPool::<T>::try_mutate(
				market,
				|opt_market_info: &mut Option<MarketInfo<T>>| -> Result<(), Error<T>> {
					match opt_market_info.as_mut() {
						Some(market_info) => {
							market_info.base_balance = market_info
								.base_balance
								.checked_sub(receive_amount)
								.ok_or(Error::<T>::Arithmetic)?;
							market_info.quote_balance = market_info
								.quote_balance
								.checked_add(deposit_amount)
								.ok_or(Error::<T>::Arithmetic)?;
							market_info.collected_quote_fees = market_info
								.collected_quote_fees
								.checked_add(fee_quote)
								.ok_or(Error::<T>::Arithmetic)?;
						},
						None => panic!("It has been checked before that this is Some; qed"),
					}

					Ok(())
				},
			)?;

			Self::deposit_event(Event::Bought(who, market, quote_amount, receive_amount));

			Ok(())
		}

		/// Allows the user to sell the BASE asset of a market
		///
		/// # Arguments:
		/// origin: The obiquitous origin of a transaction
		/// market: The market in which the user wants to trade
		/// base_amount: The amount of BASE asset the user wants to sell
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		#[transactional] // This Dispatchable is atomic
		pub fn sell(
			origin: OriginFor<T>,
			market: Market<T>,
			base_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// get balance of pool, if it exists
			let market_info =
				LiquidityPool::<T>::get(market).ok_or(Error::<T>::MarketDoesNotExist)?;

			let (base_asset, quote_asset) = market;

			// Check that user has enough BASE asset to sell it
			let base_balance = Self::balance(base_asset, &who);
			ensure!(base_balance >= base_amount, Error::<T>::NotEnoughBalance);

			let receive_amount = Self::get_received_amount(
				market_info.base_balance,
				market_info.quote_balance,
				OrderType::Sell,
				base_amount,
			)?;
			let fee_base = Self::fee_from_amount(base_amount)?;
			// This is the amount of BASE currency being deposited into the pool
			let deposit_amount = base_amount.checked_sub(fee_base).ok_or(Error::<T>::Arithmetic)?;

			let pool_account = Self::pool_account();

			// Transfer the BASE asset into the pool
			<T as Config>::Currencies::transfer(
				base_asset,
				&who,
				&pool_account,
				deposit_amount,
				true,
			)?;
			// And get the QUOTE asset out of the pool
			<T as Config>::Currencies::transfer(
				quote_asset,
				&pool_account,
				&who,
				receive_amount,
				true,
			)?;

			// Transfer taker fee into separate pool account
			let pool_fee_account = Self::pool_fee_account();
			<T as Config>::Currencies::transfer(
				base_asset,
				&who,
				&pool_fee_account,
				fee_base,
				true,
			)?;

			// update the market_info
			let fee_base = Self::fee_from_amount(base_amount)?;
			LiquidityPool::<T>::try_mutate(
				market,
				|opt_market_info: &mut Option<MarketInfo<T>>| -> Result<(), Error<T>> {
					match opt_market_info.as_mut() {
						Some(market_info) => {
							market_info.base_balance = market_info
								.base_balance
								.checked_add(deposit_amount)
								.ok_or(Error::<T>::Arithmetic)?;
							market_info.quote_balance = market_info
								.quote_balance
								.checked_sub(receive_amount)
								.ok_or(Error::<T>::Arithmetic)?;
							market_info.collected_base_fees = market_info
								.collected_base_fees
								.checked_add(fee_base)
								.ok_or(Error::<T>::Arithmetic)?;
						},
						None => panic!("It has been checked before that this is Some; qed"),
					}

					Ok(())
				},
			)?;

			Self::deposit_event(Event::Sold(who, market, base_amount, receive_amount));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// The internal account of the pool derived from this pallets id
	#[inline(always)]
	fn pool_account() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// A separate account for collecting the fees into
	#[inline(always)]
	fn pool_fee_account() -> T::AccountId {
		T::PalletId::get().try_into_sub_account(b"fee-account").expect("")
	}

	/// Calculates the received amount when buying or selling a given amount
	///
	/// # Arguments:
	/// pool_base_balance: The amount of the BASE asset in the pool
	/// pool_quote_balance: The amount of the QUOTE asset in the pool
	/// buy_or_sell: Whether the operation is buying or selling
	/// amount: The amount to spend
	///
	/// # Returns:
	/// If Ok, The balance that the user will receive from this exchange
	/// Else some arithmetic error
	fn get_received_amount(
		pool_base_balance: BalanceOf<T>,
		pool_quote_balance: BalanceOf<T>,
		buy_or_sell: OrderType,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		if amount.is_zero() {
			Ok(Zero::zero())
		} else {
			let pool_k = pool_base_balance
				.checked_mul(pool_quote_balance)
				.ok_or(Error::<T>::Arithmetic)?;

			let fee_amount = Self::fee_from_amount(amount)?;
			let amount = amount.checked_sub(fee_amount).ok_or(Error::<T>::Arithmetic)?;
			let receive_amount = match buy_or_sell {
				OrderType::Buy => {
					let new_quote_balance =
						pool_quote_balance.checked_add(amount).ok_or(Error::<T>::Arithmetic)?;
					let new_base_balance =
						pool_k.checked_div(new_quote_balance).ok_or(Error::<T>::Arithmetic)?;
					pool_base_balance.checked_sub(new_base_balance).ok_or(Error::<T>::Arithmetic)?
				},
				OrderType::Sell => {
					let new_base_balance =
						pool_base_balance.checked_add(amount).ok_or(Error::<T>::Arithmetic)?;
					let new_quote_balance =
						pool_k.checked_div(new_base_balance).ok_or(Error::<T>::Arithmetic)?;
					pool_quote_balance
						.checked_sub(new_quote_balance)
						.ok_or(Error::<T>::Arithmetic)?
				},
			};

			Ok(receive_amount)
		}
	}

	/// Helper function to get the account balance easily
	///
	/// # Arguments:
	/// asset_id: The asset were trying to query
	/// who: The account for which the balance should be retrived
	///
	/// # Returns:
	/// The balance of a user for a given asset
	///
	/// # Weight:
	/// This function has a DB read weight of 1, as it retreives the balance
	fn balance(asset_id: AssetIdOf<T>, who: &T::AccountId) -> BalanceOf<T> {
		<<T as Config>::Currencies as Inspect<<T as frame_system::Config>::AccountId>>::balance(
			asset_id, who,
		)
	}

	/// Computes the fee amount
	///
	/// # Arguments:
	/// amount: The amount to exchange from which the fees are deducted
	///
	/// # Returns:
	/// If ok, the fee amount
	/// Else the arithmetic error
	fn fee_from_amount(amount: BalanceOf<T>) -> Result<BalanceOf<T>, Error<T>> {
		let (fee_numerator, fee_denominator) = <T as Config>::TakerFee::get();

		let a = amount
			.checked_mul(BalanceOf::<T>::from(fee_numerator))
			.ok_or(Error::<T>::Arithmetic)?;

		a.checked_div(BalanceOf::<T>::from(fee_denominator))
			.ok_or(Error::<T>::Arithmetic)
	}

	/// Performs the payout of collected fee to liquidity providers
	/// Triggered every 10 blocks by offchain worker
	///
	/// # Complexity:
	/// O(n^2) currently which should be improved upon
	fn do_liquidity_provider_payout() -> Result<(), Error<T>> {
		let pool_fee_account = Self::pool_fee_account();

		let lps: Vec<(Market<T>, MarketInfo<T>)> = LiquidityPool::<T>::iter().collect();

		for (market, market_info) in &lps {
			let (base_asset, quote_asset) = market;

			if market_info.collected_base_fees == Zero::zero()
				&& market_info.collected_quote_fees == Zero::zero()
			{
				continue;
			}

			let liquidity_providers: Vec<(T::AccountId, (BalanceOf<T>, BalanceOf<T>))> =
				LiqProvisionPool::<T>::iter_prefix(market).collect();
			for (account, (base_provision, quote_provision)) in &liquidity_providers {
				if *base_provision > Zero::zero() {
					// The ratio of the users provided liquidity relative to pool liquidity for the
					// BASE asset
					let payout_fraction = base_provision
						.checked_div(market_info.base_balance)
						.ok_or(Error::<T>::Arithmetic)?;
					// The payout which is a fraction of the total collected fees
					let payout = market_info
						.collected_base_fees
						.checked_mul(payout_fraction)
						.ok_or(Error::<T>::Arithmetic)?;

					// transfer payout amount from pool_fee_account to liquidity provider
					<T as Config>::Currencies::transfer(
						*base_asset,
						&pool_fee_account,
						account,
						payout,
						true,
					)
					.map_err(|_| Error::<T>::Transfer)?;
				}
				if *quote_provision > Zero::zero() {
					// similar procedure as for the BASE asset

					let payout_fraction = quote_provision
						.checked_div(market_info.quote_balance)
						.ok_or(Error::<T>::Arithmetic)?;
					let payout = market_info
						.collected_quote_fees
						.checked_mul(payout_fraction)
						.ok_or(Error::<T>::Arithmetic)?;

					// transfer payout amount from pool_fee_account to liquidity provider
					<T as Config>::Currencies::transfer(
						*quote_asset,
						&pool_fee_account,
						account,
						payout,
						true,
					)
					.map_err(|_| Error::<T>::Transfer)?;
				}
			}

			// clear collected_base_fee as they've been distributed
			LiquidityPool::<T>::mutate(market, |opt_market_info| match opt_market_info.as_mut() {
				Some(market_info) => {
					market_info.collected_base_fees = Zero::zero();
					market_info.collected_quote_fees = Zero::zero();
				},
				None => log::error!(
					"this should not happen ever, as we previously got the key from the map; qed"
				),
			});
		}

		Ok(())
	}
}
