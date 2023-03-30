// This file is part of Polkadex.

// Copyright (C) 2020-2022 Polkadex oü.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::Get,
	traits::{fungibles::Mutate, Currency, ExistenceRequirement, OneSessionHandler},
	BoundedVec,
};
use frame_system::{ensure_signed, offchain::SubmitTransaction};
use polkadex_primitives::{assets::AssetId, AccountId, OnChainEventsLimit};
use sp_runtime::traits::Zero;

use pallet_timestamp::{self as timestamp};
use sp_core::H256;
use sp_runtime::traits::{AccountIdConversion, UniqueSaturatedInto};
use sp_std::prelude::*;
// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// ToDo: Issue 682
// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;

use orderbook_primitives::{crypto::AuthorityId, SnapshotSummary, ValidatorSet};
// ToDo: Issue 683
// #[cfg(feature = "runtime-benchmarks")]
// use sp_runtime::traits::One;
//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
pub mod weights;

pub use weights::*;

/// A type alias for the balance type from this pallet's point of view.
type BalanceOf<T> =
	<<T as Config>::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const DEPOSIT_MAX: u128 = 1_000_000_000_000_000_000_000_000_000;
const WITHDRAWAL_MAX: u128 = 1_000_000_000_000_000_000_000_000_000;
const TRADE_OPERATION_MIN_VALUE: u128 = 10000;

// Definition of the pallet logic, to be aggregated at runtime definition through
// `construct_runtime`.
#[allow(clippy::too_many_arguments)]
#[frame_support::pallet]
pub mod pallet {
	use core::fmt::Debug;
	// Import various types used to declare pallet in scope.
	use super::*;
	use bls_primitives::Signature;
	use frame_support::{
		pallet_prelude::*,
		sp_tracing::debug,
		storage::bounded_btree_map::BoundedBTreeMap,
		traits::{
			fungibles::{Create, Inspect, Mutate},
			Currency, ReservableCurrency,
		},
		PalletId,
	};
	use frame_system::{offchain::SendTransactionTypes, pallet_prelude::*};
	use liquidity::LiquidityModifier;
	use orderbook_primitives::{crypto::AuthorityId, SnapshotSummary};
	use polkadex_primitives::{
		assets::AssetId,
		ocex::{AccountInfo, TradingPairConfig},
		snapshot::{EnclaveSnapshot, Fees},
		withdrawal::Withdrawal,
		AssetsLimit, ProxyLimit, SnapshotAccLimit, WithdrawalLimit, UNIT_BALANCE,
	};
	use rust_decimal::{prelude::ToPrimitive, Decimal};
	use sp_core::H256;
	use sp_runtime::{
		traits::{IdentifyAccount, Verify},
		BoundedBTreeSet, SaturatedConversion,
	};
	use sp_std::vec::Vec;

	pub trait OcexWeightInfo {
		fn register_main_account(_b: u32) -> Weight;
		fn add_proxy_account(x: u32) -> Weight;
		fn close_trading_pair(_x: u32) -> Weight;
		fn open_trading_pair(_x: u32) -> Weight;
		fn register_trading_pair(_x: u32) -> Weight;
		fn update_trading_pair(_x: u32) -> Weight;
		fn deposit(_x: u32) -> Weight;
		fn remove_proxy_account(x: u32) -> Weight;
		fn submit_snapshot() -> Weight;
		fn insert_enclave(_x: u32) -> Weight;
		fn collect_fees(_x: u32) -> Weight;
		fn shutdown() -> Weight;
		fn set_exchange_state(_x: u32) -> Weight;
		fn set_balances(_x: u32) -> Weight;
		fn claim_withdraw(_x: u32) -> Weight;
		fn register_enclave(_x: u32) -> Weight;
		fn allowlist_token(_x: u32) -> Weight;
		fn remove_allowlisted_token(_x: u32) -> Weight;
		fn allowlist_enclave(_x: u32) -> Weight;
		fn update_certificate(_x: u32) -> Weight;
	}

	type WithdrawalsMap<T> = BoundedBTreeMap<
		<T as frame_system::Config>::AccountId,
		BoundedVec<Withdrawal<<T as frame_system::Config>::AccountId>, WithdrawalLimit>,
		SnapshotAccLimit,
	>;

	type EnclaveSnapshotType<T> = EnclaveSnapshot<
		<T as frame_system::Config>::AccountId,
		WithdrawalLimit,
		AssetsLimit,
		SnapshotAccLimit,
	>;

	pub struct AllowlistedTokenLimit;
	impl Get<u32> for AllowlistedTokenLimit {
		fn get() -> u32 {
			50 // TODO: Arbitrary value
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> frame_support::unsigned::ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			sp_runtime::print("Entering validate unsigned....");
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("orderbook")
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			let validate_snapshot = |snapshot_summary: &SnapshotSummary| -> TransactionValidity {
				// Verify Nonce/state_change_id
				let last_snapshot_serial_number = <SnapshotNonce<T>>::get();
				if !snapshot_summary.state_change_id.eq(&(last_snapshot_serial_number + 1)) {
					return InvalidTransaction::Custom(10).into()
				}

				// Get authority from active set
				// index is zero because we are signing only with one authority
				// when submitting snapshot
				let auth_idx = match snapshot_summary.signed_auth_indexes().get(0) {
					Some(idx) => *idx,
					None => return InvalidTransaction::BadSigner.into(),
				};
				let authority = match <Authorities<T>>::get().get(auth_idx as usize) {
					Some(auth) => auth,
					None => return InvalidTransaction::Custom(11).into(),
				}
				.clone();

				// Verify Signature
				match snapshot_summary.aggregate_signature {
					None => return InvalidTransaction::Custom(12).into(),
					Some(signature) => {
						// TODO: Ivan to fix this.
						// if !bls_primitives::crypto::bls_ext::verify(
						// 	&authority.into(),
						// 	&snapshot_summary.sign_data(),
						// 	&signature,
						// ) {
						// 	return InvalidTransaction::Custom(13).into()
						// }
					},
				}
				sp_runtime::print("Signature successfull");
				valid_tx(snapshot_summary.clone())
			};
			match call {
				Call::submit_snapshot { summary } => validate_snapshot(summary),
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	/// Our pallet's configuration trait. All our types and constants go in here. If the
	/// pallet is dependent on specific other pallets, then their configuration traits
	/// should be added to our implied traits list.
	///
	/// `frame_system::Config` should always be included.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + timestamp::Config + SendTransactionTypes<Call<Self>>
	{
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Address which holds the customer funds.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Balances Pallet
		type NativeCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Assets Pallet
		type OtherAssets: Mutate<
				<Self as frame_system::Config>::AccountId,
				Balance = BalanceOf<Self>,
				AssetId = u128,
			> + Inspect<<Self as frame_system::Config>::AccountId>
			+ Create<<Self as frame_system::Config>::AccountId>;

		/// Origin that can send orderbook snapshots and withdrawal requests
		type EnclaveOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;
		type Public: Clone
			+ PartialEq
			+ IdentifyAccount<AccountId = Self::AccountId>
			+ Debug
			+ parity_scale_codec::Codec
			+ Ord
			+ scale_info::TypeInfo;

		/// A matching `Signature` type.
		type Signature: Verify<Signer = Self::Public>
			+ Clone
			+ PartialEq
			+ Debug
			+ parity_scale_codec::Codec
			+ scale_info::TypeInfo;

		/// Type representing the weight of this pallet
		type WeightInfo: OcexWeightInfo;

		// declared number of milliseconds per day and is used to determine
		// enclave's report validity time.
		// standard 24h in ms = 86_400_000
		type MsPerDay: Get<Self::Moment>;

		/// Governance Origin
		type GovernanceOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;
	}

	// Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
	// method.
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// Unable to convert given balance to internal Decimal data type
		FailedToConvertDecimaltoBalance,
		RegisterationShouldBeSignedByMainAccount,
		/// Caller is not authorized to claim the withdrawal.
		/// Normally, when Sender != main_account.
		SenderNotAuthorizedToWithdraw,
		/// Account is not registered with the exchange
		AccountNotRegistered,
		InvalidWithdrawalIndex,
		/// Amount within withdrawal can not be converted to Decimal
		InvalidWithdrawalAmount,
		/// The trading pair is not currently Operational
		TradingPairIsNotOperational,
		/// the trading pair is currently in operation
		TradingPairIsNotClosed,
		MainAccountAlreadyRegistered,
		SnapshotNonceError,
		EnclaveSignatureVerificationFailed,
		MainAccountNotFound,
		ProxyLimitExceeded,
		TradingPairAlreadyRegistered,
		BothAssetsCannotBeSame,
		TradingPairNotFound,
		/// Provided Report Value is invalid
		InvalidReportValue,
		/// IAS attestation verification failed:
		/// a) certificate[s] outdated;
		/// b) enclave is not properly signed it's report with IAS service;
		RemoteAttestationVerificationFailed,
		/// Sender has not been attested
		SenderIsNotAttestedEnclave,
		/// RA status is insufficient
		InvalidSgxReportStatus,
		/// Storage overflow ocurred
		AmountOverflow,
		///ProxyNotFound
		ProxyNotFound,
		/// MinimumOneProxyRequried
		MinimumOneProxyRequired,
		/// Onchain Events vector is full
		OnchainEventsBoundedVecOverflow,
		/// Overflow of Deposit amount
		DepositOverflow,
		/// Enclave not allowlisted
		EnclaveNotAllowlisted,
		/// Trading Pair is not registed for updating
		TradingPairNotRegistered,
		/// Trading Pair config value cannot be set to zero
		TradingPairConfigCannotBeZero,
		/// Limit reached to add allowlisted token
		AllowlistedTokenLimitReached,
		/// Given token is not allowlisted
		TokenNotAllowlisted,
		/// Given allowlisted token is removed
		AllowlistedTokenRemoved,
		/// Trading Pair config value cannot be set to zero
		TradingPairConfigUnderflow,
		/// Exchange is down
		ExchangeNotOperational,
		/// Unable to transfer fee
		UnableToTransferFee,
		/// Unable to execute collect fees fully
		FeesNotCollectedFully,
		/// Exchange is up
		ExchangeOperational,
		/// Can not write into withdrawal bounded structure
		/// limit reached
		WithdrawalBoundOverflow,
		/// Unable to aggregrate the signature
		InvalidSignatureAggregation,
		/// Unable to get signer index
		SignerIndexNotFound,
		/// Snapshot in invalid state
		InvalidSnapshotState,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// What to do at the end of each block.
		///
		/// Clean IngressMessages
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			<IngressMessages<T>>::put(Vec::<
				polkadex_primitives::ingress::IngressMessages<T::AccountId>,
			>::new());

			<OnChainEvents<T>>::put(BoundedVec::<
				polkadex_primitives::ocex::OnChainEvents<T::AccountId>,
				OnChainEventsLimit,
			>::default());

			(1000000 as Weight)
				.saturating_add(T::DbWeight::get().reads(2 as Weight))
				.saturating_add(T::DbWeight::get().writes(2 as Weight))
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registers a new account in orderbook
		#[pallet::weight(<T as Config>::WeightInfo::register_main_account(1))]
		pub fn register_main_account(origin: OriginFor<T>, proxy: T::AccountId) -> DispatchResult {
			let main_account = ensure_signed(origin)?;
			Self::register_user(main_account, proxy)?;
			Ok(())
		}

		/// Adds a proxy account to a pre-registered main acocunt
		#[pallet::weight(<T as Config>::WeightInfo::add_proxy_account(1))]
		pub fn add_proxy_account(origin: OriginFor<T>, proxy: T::AccountId) -> DispatchResult {
			let main_account = ensure_signed(origin)?;
			ensure!(Self::orderbook_operational_state(), Error::<T>::ExchangeNotOperational);
			ensure!(<Accounts<T>>::contains_key(&main_account), Error::<T>::MainAccountNotFound);
			if let Some(mut account_info) = <Accounts<T>>::get(&main_account) {
				ensure!(
					account_info.add_proxy(proxy.clone()).is_ok(),
					Error::<T>::ProxyLimitExceeded
				);

				<IngressMessages<T>>::mutate(|ingress_messages| {
					ingress_messages.push(polkadex_primitives::ingress::IngressMessages::AddProxy(
						main_account.clone(),
						proxy.clone(),
					));
				});
				<Accounts<T>>::insert(&main_account, account_info);
				Self::deposit_event(Event::MainAccountRegistered { main: main_account, proxy });
			}
			Ok(())
		}

		/// Registers a new trading pair
		#[pallet::weight(<T as Config>::WeightInfo::close_trading_pair(1))]
		pub fn close_trading_pair(
			origin: OriginFor<T>,
			base: AssetId,
			quote: AssetId,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			ensure!(Self::orderbook_operational_state(), Error::<T>::ExchangeNotOperational);
			ensure!(base != quote, Error::<T>::BothAssetsCannotBeSame);
			ensure!(<TradingPairs<T>>::contains_key(base, quote), Error::<T>::TradingPairNotFound);
			<TradingPairs<T>>::mutate(base, quote, |value| {
				if let Some(trading_pair) = value {
					trading_pair.operational_status = false;
					<IngressMessages<T>>::mutate(|ingress_messages| {
						ingress_messages.push(
							polkadex_primitives::ingress::IngressMessages::CloseTradingPair(
								trading_pair.clone(),
							),
						);
					});
					Self::deposit_event(Event::ShutdownTradingPair { pair: trading_pair.clone() });
				} else {
					//scope never executed, already ensured if trading pair exits above
				}
			});
			Ok(())
		}

		/// Registers a new trading pair
		#[pallet::weight(<T as Config>::WeightInfo::open_trading_pair(1))]
		pub fn open_trading_pair(
			origin: OriginFor<T>,
			base: AssetId,
			quote: AssetId,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			ensure!(Self::orderbook_operational_state(), Error::<T>::ExchangeNotOperational);
			ensure!(base != quote, Error::<T>::BothAssetsCannotBeSame);
			ensure!(<TradingPairs<T>>::contains_key(base, quote), Error::<T>::TradingPairNotFound);
			//update the operational status of the trading pair as true.
			<TradingPairs<T>>::mutate(base, quote, |value| {
				if let Some(trading_pair) = value {
					trading_pair.operational_status = true;
					<IngressMessages<T>>::mutate(|ingress_messages| {
						ingress_messages.push(
							polkadex_primitives::ingress::IngressMessages::OpenTradingPair(
								trading_pair.clone(),
							),
						);
					});
					Self::deposit_event(Event::OpenTradingPair { pair: trading_pair.clone() });
				} else {
					//scope never executed, already ensured if trading pair exits above
				}
			});
			Ok(())
		}

		/// Registers a new trading pair
		#[pallet::weight(<T as Config>::WeightInfo::register_trading_pair(1))]
		pub fn register_trading_pair(
			origin: OriginFor<T>,
			base: AssetId,
			quote: AssetId,
			min_order_price: BalanceOf<T>,
			max_order_price: BalanceOf<T>,
			min_order_qty: BalanceOf<T>,
			max_order_qty: BalanceOf<T>,
			price_tick_size: BalanceOf<T>,
			qty_step_size: BalanceOf<T>,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			ensure!(Self::orderbook_operational_state(), Error::<T>::ExchangeNotOperational);

			ensure!(base != quote, Error::<T>::BothAssetsCannotBeSame);
			ensure!(
				!<TradingPairs<T>>::contains_key(base, quote),
				Error::<T>::TradingPairAlreadyRegistered
			);
			ensure!(
				!<TradingPairs<T>>::contains_key(quote, base),
				Error::<T>::TradingPairAlreadyRegistered
			);

			// We need to also check if provided values are not zero
			ensure!(
				min_order_price.saturated_into::<u128>() > 0 &&
					max_order_price.saturated_into::<u128>() > 0 &&
					min_order_qty.saturated_into::<u128>() > 0 &&
					max_order_qty.saturated_into::<u128>() > 0 &&
					price_tick_size.saturated_into::<u128>() > 0 &&
					qty_step_size.saturated_into::<u128>() > 0,
				Error::<T>::TradingPairConfigCannotBeZero
			);

			// We need to check if the provided parameters are not exceeding 10^27 so that there
			// will not be an overflow upon performing calculations
			ensure!(
				min_order_price.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				max_order_price.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				min_order_qty.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				max_order_qty.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				price_tick_size.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				qty_step_size.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);

			//enclave will only support min volume of 10^-8
			//if trading pairs volume falls below it will pass a UnderFlow Error
			ensure!(
				min_order_price.saturated_into::<u128>() > TRADE_OPERATION_MIN_VALUE &&
					min_order_qty.saturated_into::<u128>() > TRADE_OPERATION_MIN_VALUE &&
					min_order_price
						.saturated_into::<u128>()
						.saturating_mul(min_order_qty.saturated_into::<u128>()) >
						TRADE_OPERATION_MIN_VALUE,
				Error::<T>::TradingPairConfigUnderflow
			);

			// TODO: Check if base and quote assets are enabled for deposits
			// Decimal::from() here is infallable as we ensure provided parameters do not exceed
			// Decimal::MAX
			match (
				Decimal::from(min_order_price.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(max_order_price.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(price_tick_size.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(min_order_qty.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(max_order_qty.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(qty_step_size.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
			) {
				(
					Some(min_price),
					Some(max_price),
					Some(price_tick_size),
					Some(min_qty),
					Some(max_qty),
					Some(qty_step_size),
				) => {
					let trading_pair_info = TradingPairConfig {
						base_asset: base,
						quote_asset: quote,
						min_price,
						max_price,
						price_tick_size,
						min_qty,
						max_qty,
						qty_step_size,
						operational_status: true,
						base_asset_precision: qty_step_size.scale() as u8,
						quote_asset_precision: price_tick_size.scale() as u8,
					};

					<TradingPairs<T>>::insert(base, quote, trading_pair_info.clone());
					<IngressMessages<T>>::mutate(|ingress_messages| {
						ingress_messages.push(
							polkadex_primitives::ingress::IngressMessages::OpenTradingPair(
								trading_pair_info,
							),
						);
					});
					Self::deposit_event(Event::TradingPairRegistered { base, quote });
					Ok(())
				},
				//passing Underflow error if checked_div fails
				_ => Err(Error::<T>::TradingPairConfigUnderflow.into()),
			}
		}

		/// Updates the trading pair config
		#[pallet::weight(<T as Config>::WeightInfo::update_trading_pair(1))]
		pub fn update_trading_pair(
			origin: OriginFor<T>,
			base: AssetId,
			quote: AssetId,
			min_order_price: BalanceOf<T>,
			max_order_price: BalanceOf<T>,
			min_order_qty: BalanceOf<T>,
			max_order_qty: BalanceOf<T>,
			price_tick_size: BalanceOf<T>,
			qty_step_size: BalanceOf<T>,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			ensure!(Self::orderbook_operational_state(), Error::<T>::ExchangeNotOperational);
			ensure!(base != quote, Error::<T>::BothAssetsCannotBeSame);
			ensure!(
				<TradingPairs<T>>::contains_key(base, quote),
				Error::<T>::TradingPairNotRegistered
			);
			let is_pair_in_operation = match <TradingPairs<T>>::get(base, quote) {
				Some(config) => config.operational_status,
				None => false,
			};
			ensure!(!is_pair_in_operation, Error::<T>::TradingPairIsNotClosed);
			// We need to also check if provided values are not zero
			ensure!(
				min_order_price.saturated_into::<u128>() > 0 &&
					max_order_price.saturated_into::<u128>() > 0 &&
					min_order_qty.saturated_into::<u128>() > 0 &&
					max_order_qty.saturated_into::<u128>() > 0 &&
					price_tick_size.saturated_into::<u128>() > 0 &&
					qty_step_size.saturated_into::<u128>() > 0,
				Error::<T>::TradingPairConfigCannotBeZero
			);

			// We need to check if the provided parameters are not exceeding 10^27 so that there
			// will not be an overflow upon performing calculations
			ensure!(
				min_order_price.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				max_order_price.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				min_order_qty.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				max_order_qty.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				price_tick_size.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);
			ensure!(
				qty_step_size.saturated_into::<u128>() <= DEPOSIT_MAX,
				Error::<T>::AmountOverflow
			);

			//enclave will only support min volume of 10^-8
			//if trading pairs volume falls below it will pass a UnderFlow Error
			ensure!(
				min_order_price.saturated_into::<u128>() > TRADE_OPERATION_MIN_VALUE &&
					min_order_qty.saturated_into::<u128>() > TRADE_OPERATION_MIN_VALUE &&
					min_order_price
						.saturated_into::<u128>()
						.saturating_mul(min_order_qty.saturated_into::<u128>()) >
						TRADE_OPERATION_MIN_VALUE,
				Error::<T>::TradingPairConfigUnderflow
			);

			match (
				Decimal::from(min_order_price.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(max_order_price.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(price_tick_size.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(min_order_qty.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(max_order_qty.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
				Decimal::from(qty_step_size.saturated_into::<u128>())
					.checked_div(Decimal::from(UNIT_BALANCE)),
			) {
				(
					Some(min_price),
					Some(max_price),
					Some(price_tick_size),
					Some(min_qty),
					Some(max_qty),
					Some(qty_step_size),
				) => {
					let trading_pair_info = TradingPairConfig {
						base_asset: base,
						quote_asset: quote,
						min_price,
						max_price,
						price_tick_size,
						min_qty,
						max_qty,
						qty_step_size,
						operational_status: true,
						base_asset_precision: price_tick_size.scale() as u8, /* scale() can never be                                                    * greater u8::MAX */
						quote_asset_precision: qty_step_size.scale() as u8, /* scale() can never be                                                    * greater than u8::MAX */
					};

					<TradingPairs<T>>::insert(base, quote, trading_pair_info.clone());
					<IngressMessages<T>>::mutate(|ingress_messages| {
						ingress_messages.push(
							polkadex_primitives::ingress::IngressMessages::UpdateTradingPair(
								trading_pair_info,
							),
						);
					});
					Self::deposit_event(Event::TradingPairUpdated { base, quote });

					Ok(())
				},
				_ => Err(Error::<T>::TradingPairConfigUnderflow.into()),
			}
		}

		/// Deposit Assets to Orderbook
		#[pallet::weight(<T as Config>::WeightInfo::deposit(1))]
		pub fn deposit(
			origin: OriginFor<T>,
			asset: AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;
			Self::do_deposit(user, asset, amount)?;
			Ok(())
		}

		/// Removes a proxy account from pre-registered main account
		#[pallet::weight(<T as Config>::WeightInfo::remove_proxy_account(1))]
		pub fn remove_proxy_account(origin: OriginFor<T>, proxy: T::AccountId) -> DispatchResult {
			let main_account = ensure_signed(origin)?;
			ensure!(Self::orderbook_operational_state(), Error::<T>::ExchangeNotOperational);
			ensure!(<Accounts<T>>::contains_key(&main_account), Error::<T>::MainAccountNotFound);
			<Accounts<T>>::try_mutate(&main_account, |account_info| {
				if let Some(account_info) = account_info {
					ensure!(account_info.proxies.len() > 1, Error::<T>::MinimumOneProxyRequired);
					let proxy_positon = account_info
						.proxies
						.iter()
						.position(|account| *account == proxy)
						.ok_or(Error::<T>::ProxyNotFound)?;
					account_info.proxies.remove(proxy_positon);
					<IngressMessages<T>>::mutate(|ingress_messages| {
						ingress_messages.push(
							polkadex_primitives::ingress::IngressMessages::RemoveProxy(
								main_account.clone(),
								proxy.clone(),
							),
						);
					});
				}
				Self::deposit_event(Event::ProxyRemoved { main: main_account.clone(), proxy });
				Ok(())
			})
		}

		//TODO: Benchmark set_snapshot
		#[pallet::weight(<T as Config>::WeightInfo::submit_snapshot())]
		pub fn set_snapshot(origin: OriginFor<T>, new_snapshot_id: u64) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			<SnapshotNonce<T>>::put(new_snapshot_id);
			Ok(())
		}

		// TODO: Benchmark change_pending_withdrawal_limit (Issue number 683)
		/// The extrinsic will be used to change pending withdrawals limit
		///
		/// # Parameters
		/// * `origin`: Orderbook governance
		/// * `new_pending_withdrawals_limit`: The new pending withdrawals limit governance
		/// wants to set.
		#[pallet::weight(<T as Config>::WeightInfo::submit_snapshot())]
		pub fn change_pending_withdrawal_limit(
			origin: OriginFor<T>,
			new_pending_withdrawals_limit: u64,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			<PendingWithdrawalsAllowedPerSnapshot<T>>::put(new_pending_withdrawals_limit);
			Ok(())
		}

		// TODO: Benchmark change_snapshot_interval_block (Issue number 683)
		/// The extrinsic will be used to change snapshot interval based on block number
		///
		/// # Parameters
		/// * `origin`: Orderbook governance
		/// * `new_snapshot_interval_block`: The new block interval at which snapshot should  be
		/// generated.
		#[pallet::weight(<T as Config>::WeightInfo::submit_snapshot())]
		pub fn change_snapshot_interval_block(
			origin: OriginFor<T>,
			new_snapshot_interval_block: T::BlockNumber,
		) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			<SnapshotIntervalBlock<T>>::put(new_snapshot_interval_block);
			Ok(())
		}

		/// Withdraws Fees Collected
		///
		/// params:  snapshot_number: u32
		#[pallet::weight(<T as Config>::WeightInfo::collect_fees(1))]
		pub fn collect_fees(
			origin: OriginFor<T>,
			snapshot_id: u32,
			beneficiary: T::AccountId,
		) -> DispatchResult {
			// TODO: The caller should be of operational council
			T::GovernanceOrigin::ensure_origin(origin)?;

			ensure!(
				<FeesCollected<T>>::mutate(snapshot_id, |internal_vector| {
					while internal_vector.len() > 0 {
						if let Some(fees) = internal_vector.pop() {
							if let Some(converted_fee) =
								fees.amount.saturating_mul(Decimal::from(UNIT_BALANCE)).to_u128()
							{
								if Self::transfer_asset(
									&Self::get_pallet_account(),
									&beneficiary,
									converted_fee.saturated_into(),
									fees.asset,
								)
								.is_err()
								{
									// Push it back inside the internal vector
									// The above function call will only fail if the beneficiary has
									// balance below existential deposit requirements
									internal_vector.try_push(fees).unwrap_or_default();
									return Err(Error::<T>::UnableToTransferFee)
								}
							} else {
								// Push it back inside the internal vector
								internal_vector.try_push(fees).unwrap_or_default();
								return Err(Error::<T>::FailedToConvertDecimaltoBalance)
							}
						}
					}
					Ok(())
				})
				.is_ok(),
				Error::<T>::FeesNotCollectedFully
			);
			Self::deposit_event(Event::FeesClaims { beneficiary, snapshot_id });
			Ok(())
		}

		/// Extrinsic used to shutdown the orderbook
		#[pallet::weight(<T as Config>::WeightInfo::shutdown())]
		pub fn shutdown(origin: OriginFor<T>) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			<ExchangeState<T>>::put(false);
			<IngressMessages<T>>::mutate(|ingress_messages| {
				ingress_messages.push(polkadex_primitives::ingress::IngressMessages::Shutdown);
			});
			Ok(())
		}

		///This extrinsic will pause/resume the exchange according to flag
		/// If flag is set to false it will stop the exchange
		/// If flag is set to true it will resume the exchange
		#[pallet::weight(<T as Config>::WeightInfo::set_exchange_state(1))]
		pub fn set_exchange_state(origin: OriginFor<T>, state: bool) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			<ExchangeState<T>>::put(state);

			//SetExchangeState Ingress message store in queue
			<IngressMessages<T>>::mutate(|ingress_messages| {
				ingress_messages
					.push(polkadex_primitives::ingress::IngressMessages::SetExchangeState(state))
			});

			Self::deposit_event(Event::ExchangeStateUpdated(state));
			Ok(())
		}

		/// Sends the changes required in balances for list of users with a particular asset
		#[pallet::weight(<T as Config>::WeightInfo::set_balances(change_in_balances.len().saturated_into()))]
		pub fn set_balances(
			origin: OriginFor<T>,
			change_in_balances: BoundedVec<
				polkadex_primitives::ingress::HandleBalance<T::AccountId>,
				polkadex_primitives::ingress::HandleBalanceLimit,
			>,
		) -> DispatchResult {
			// Check if governance called the extrinsic
			T::GovernanceOrigin::ensure_origin(origin)?;

			// Check if exchange is pause
			ensure!(!Self::orderbook_operational_state(), Error::<T>::ExchangeOperational);

			//Pass the vec as ingress message
			<IngressMessages<T>>::mutate(|ingress_messages| {
				ingress_messages.push(
					polkadex_primitives::ingress::IngressMessages::SetFreeReserveBalanceForAccounts(
						change_in_balances,
					),
				);
			});
			Ok(())
		}

		/// Withdraws user balance
		///
		/// params: snapshot_number: u32
		/// account: AccountId
		#[pallet::weight(<T as Config>::WeightInfo::claim_withdraw(1))]
		pub fn claim_withdraw(
			origin: OriginFor<T>,
			snapshot_id: u32,
			account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			// Anyone can claim the withdrawal for any user
			// This is to build services that can enable free withdrawals similar to CEXes.
			let _ = ensure_signed(origin)?;
			// This vector will keep track of withdrawals processed already
			let mut processed_withdrawals = vec![];
			let mut failed_withdrawals = vec![];
			ensure!(
				<Withdrawals<T>>::contains_key(snapshot_id),
				Error::<T>::InvalidWithdrawalIndex
			);
			// This entire block of code is put inside ensure as some of the nested functions will
			// return Err
			<Withdrawals<T>>::mutate(snapshot_id, |btree_map| {
				// Get mutable reference to the withdrawals vector
				if let Some(withdrawal_vector) = btree_map.get_mut(&account) {
					while withdrawal_vector.len() > 0 {
						// Perform pop operation to ensure we do not leave any withdrawal left
						// for a double spend
						if let Some(withdrawal) = withdrawal_vector.pop() {
							if let Some(converted_withdrawal) = withdrawal
								.amount
								.saturating_mul(Decimal::from(UNIT_BALANCE))
								.to_u128()
							{
								if Self::transfer_asset(
									&Self::get_pallet_account(),
									&withdrawal.main_account,
									converted_withdrawal.saturated_into(),
									withdrawal.asset,
								)
								.is_ok()
								{
									processed_withdrawals.push(withdrawal.to_owned());
								} else {
									// Storing the failed withdrawals back into the storage item
									failed_withdrawals.push(withdrawal.to_owned());
									Self::deposit_event(Event::WithdrawalFailed(
										withdrawal.to_owned(),
									));
								}
							} else {
								return Err(Error::<T>::InvalidWithdrawalAmount)
							}
						}
					}
					// Not removing key from BtreeMap so that failed withdrawals can still be
					// tracked
					btree_map
						.try_insert(
							account.clone(),
							failed_withdrawals
								.try_into()
								.map_err(|_| Error::<T>::WithdrawalBoundOverflow)?,
						)
						.map_err(|_| Error::<T>::WithdrawalBoundOverflow)?;
					Ok(())
				} else {
					// This allows us to ensure we do not have someone with an invalid account
					Err(Error::<T>::InvalidWithdrawalIndex)
				}
			})?;
			if !processed_withdrawals.is_empty() {
				Self::deposit_event(Event::WithdrawalClaimed {
					main: account.clone(),
					withdrawals: processed_withdrawals.clone(),
				});
				<OnChainEvents<T>>::mutate(|onchain_events| {
					onchain_events
						.try_push(
							polkadex_primitives::ocex::OnChainEvents::OrderBookWithdrawalClaimed(
								snapshot_id,
								account.clone(),
								processed_withdrawals
									.clone()
									.try_into()
									.map_err(|_| Error::<T>::WithdrawalBoundOverflow)?,
							),
						)
						.map_err(|_| Error::<T>::WithdrawalBoundOverflow)?;
					Ok::<(), Error<T>>(())
				})?;
				Ok(Pays::No.into())
			} else {
				// If someone withdraws nothing successfully - should pay for such transaction
				Ok(Pays::Yes.into())
			}
		}

		/// Allowlist Token
		#[pallet::weight(<T as Config>::WeightInfo::allowlist_token(1))]
		pub fn allowlist_token(origin: OriginFor<T>, token: AssetId) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			let mut allowlisted_tokens = <AllowlistedToken<T>>::get();
			allowlisted_tokens
				.try_insert(token)
				.map_err(|_| Error::<T>::AllowlistedTokenLimitReached)?;
			<AllowlistedToken<T>>::put(allowlisted_tokens);
			Self::deposit_event(Event::<T>::TokenAllowlisted(token));
			Ok(())
		}

		/// Remove Allowlisted Token
		#[pallet::weight(<T as Config>::WeightInfo::remove_allowlisted_token(1))]
		pub fn remove_allowlisted_token(origin: OriginFor<T>, token: AssetId) -> DispatchResult {
			T::GovernanceOrigin::ensure_origin(origin)?;
			let mut allowlisted_tokens = <AllowlistedToken<T>>::get();
			allowlisted_tokens.remove(&token);
			<AllowlistedToken<T>>::put(allowlisted_tokens);
			Self::deposit_event(Event::<T>::AllowlistedTokenRemoved(token));
			Ok(())
		}

		/// Submit Snapshot Summary
		#[pallet::weight(10000)]
		pub fn submit_snapshot(origin: OriginFor<T>, summary: SnapshotSummary) -> DispatchResult {
			ensure_none(origin)?;
			let last_snapshot_serial_number = <SnapshotNonce<T>>::get();
			ensure!(
				summary.state_change_id.eq(&(last_snapshot_serial_number + 1)),
				Error::<T>::SnapshotNonceError
			);
			let summary_hash = H256::from_slice(&summary.sign_data());
			let working_summary =
				match <UnprocessedSnapshots<T>>::get(summary.snapshot_id, summary_hash) {
					None => summary,
					Some(mut stored_summary) => {
						if let Some(signature) = summary.aggregate_signature {
							// Aggregrate the signature
							if let Err(_) = stored_summary.add_signature(signature) {
								return Err(Error::<T>::InvalidSignatureAggregation.into())
							}
							// update the bitfield
							let auth_index = match summary.signed_auth_indexes().get(0) {
								Some(index) => *index,
								None => return Err(Error::<T>::SignerIndexNotFound.into()),
							};
							stored_summary.add_auth_index(auth_index);
							stored_summary
						} else {
							return Err(Error::<T>::InvalidSnapshotState.into())
						}
					},
				};
			// Check if we have enough signatures
			let total_validators = <Authorities<T>>::get().len();
			if working_summary.signed_auth_indexes().len() >=
				total_validators.saturating_mul(2).saturating_div(3)
			{
				// TODO: Do we need to verify aggregrate signature once more???
				// Remove all the unprocessed snapshots with prefix snapshot_id
				let mut result = <UnprocessedSnapshots<T>>::clear_prefix(
					working_summary.snapshot_id,
					total_validators as u32,
					None,
				);
				while result.maybe_cursor.is_some() {
					result = <UnprocessedSnapshots<T>>::clear_prefix(
						working_summary.snapshot_id,
						total_validators as u32,
						Some(result.maybe_cursor.unwrap().as_ref()),
					);
				}
				// Update the snapshot nonce and move the summary to snapshots storage
				<SnapshotNonce<T>>::put(working_summary.snapshot_id);
				<Snapshots<T>>::insert(working_summary.snapshot_id, working_summary);
				// Clear PendingSnapshotFromPreviousSet storage if its present
				// because we are accepted this snapshot as there are not pending snapshots
				<PendingSnapshotFromPreviousSet<T>>::kill();
			}
			Ok(())
		}
	}

	impl<T: Config> LiquidityModifier for Pallet<T> {
		type AssetId = AssetId;
		type AccountId = T::AccountId;

		fn on_deposit(
			account: Self::AccountId,
			asset: Self::AssetId,
			balance: u128,
		) -> DispatchResult {
			Self::do_deposit(account, asset, balance.saturated_into())?;
			Ok(())
		}
		fn on_withdraw(
			account: Self::AccountId,
			proxy_account: Self::AccountId,
			asset: Self::AssetId,
			balance: u128,
			do_force_withdraw: bool,
		) -> DispatchResult {
			Self::withdrawal_from_orderbook(
				account,
				proxy_account,
				asset,
				balance.saturated_into(),
				do_force_withdraw,
			)?;
			Ok(())
		}
		fn on_register(main_account: Self::AccountId, proxy: Self::AccountId) -> DispatchResult {
			Self::register_user(main_account, proxy)?;
			Ok(())
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn set_exchange_state_to_true() -> DispatchResult {
			<ExchangeState<T>>::put(true);
			Ok(())
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn allowlist_and_create_token(account: Self::AccountId, token: u128) -> DispatchResult {
			let asset: AssetId = AssetId::asset(token);
			let mut allowlisted_tokens = <AllowlistedToken<T>>::get();
			allowlisted_tokens
				.try_insert(asset)
				.map_err(|_| Error::<T>::AllowlistedTokenLimitReached)?;
			<AllowlistedToken<T>>::put(allowlisted_tokens);
			let amount = BalanceOf::<T>::decode(&mut &(u128::MAX).to_le_bytes()[..])
				.map_err(|_| Error::<T>::FailedToConvertDecimaltoBalance)?;
			//create asset and mint into it.
			T::OtherAssets::create(
				token,
				Self::get_pallet_account(),
				true,
				BalanceOf::<T>::one().unique_saturated_into(),
			)?;
			T::OtherAssets::mint_into(token, &account.clone(), amount)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn do_deposit(
			user: T::AccountId,
			asset: AssetId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure!(Self::orderbook_operational_state(), Error::<T>::ExchangeNotOperational);
			ensure!(<AllowlistedToken<T>>::get().contains(&asset), Error::<T>::TokenNotAllowlisted);
			// Check if account is registered
			ensure!(<Accounts<T>>::contains_key(&user), Error::<T>::AccountNotRegistered);
			ensure!(amount.saturated_into::<u128>() <= DEPOSIT_MAX, Error::<T>::AmountOverflow);
			let converted_amount = Decimal::from(amount.saturated_into::<u128>())
				.checked_div(Decimal::from(UNIT_BALANCE))
				.ok_or(Error::<T>::FailedToConvertDecimaltoBalance)?;

			Self::transfer_asset(&user, &Self::get_pallet_account(), amount, asset)?;
			// Get Storage Map Value
			if let Some(expected_total_amount) =
				converted_amount.checked_add(Self::total_assets(asset))
			{
				<TotalAssets<T>>::insert(asset, expected_total_amount);
			} else {
				return Err(Error::<T>::AmountOverflow.into())
			}

			<IngressMessages<T>>::mutate(|ingress_messages| {
				ingress_messages.push(polkadex_primitives::ingress::IngressMessages::Deposit(
					user.clone(),
					asset,
					converted_amount,
				));
			});
			Self::deposit_event(Event::DepositSuccessful { user, asset, amount });
			Ok(())
		}

		pub fn register_user(main_account: T::AccountId, proxy: T::AccountId) -> DispatchResult {
			ensure!(Self::orderbook_operational_state(), Error::<T>::ExchangeNotOperational);
			ensure!(
				!<Accounts<T>>::contains_key(&main_account),
				Error::<T>::MainAccountAlreadyRegistered
			);

			let mut account_info = AccountInfo::new(main_account.clone());
			ensure!(account_info.add_proxy(proxy.clone()).is_ok(), Error::<T>::ProxyLimitExceeded);
			<Accounts<T>>::insert(&main_account, account_info);

			<IngressMessages<T>>::mutate(|ingress_messages| {
				ingress_messages.push(polkadex_primitives::ingress::IngressMessages::RegisterUser(
					main_account.clone(),
					proxy.clone(),
				));
			});
			Self::deposit_event(Event::MainAccountRegistered { main: main_account, proxy });
			Ok(())
		}

		pub fn withdrawal_from_orderbook(
			user: T::AccountId,
			proxy_account: T::AccountId,
			asset: AssetId,
			amount: BalanceOf<T>,
			do_force_withdraw: bool,
		) -> DispatchResult {
			ensure!(Self::orderbook_operational_state(), Error::<T>::ExchangeNotOperational);
			ensure!(<AllowlistedToken<T>>::get().contains(&asset), Error::<T>::TokenNotAllowlisted);
			// Check if account is registered
			ensure!(<Accounts<T>>::contains_key(&user), Error::<T>::AccountNotRegistered);
			ensure!(amount.saturated_into::<u128>() <= WITHDRAWAL_MAX, Error::<T>::AmountOverflow);
			let converted_amount = Decimal::from(amount.saturated_into::<u128>())
				.checked_div(Decimal::from(UNIT_BALANCE))
				.ok_or(Error::<T>::FailedToConvertDecimaltoBalance)?;
			<IngressMessages<T>>::mutate(|ingress_messages| {
				ingress_messages.push(
					polkadex_primitives::ingress::IngressMessages::DirectWithdrawal(
						proxy_account,
						asset,
						converted_amount,
						do_force_withdraw,
					),
				);
			});
			Self::deposit_event(Event::WithdrawFromOrderbook(user, asset, amount));
			Ok(())
		}
	}

	/// Events are a simple means of reporting specific conditions and
	/// circumstances that have happened that users, Dapps and/or chain explorers would find
	/// interesting and otherwise difficult to detect.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		FeesClaims {
			beneficiary: T::AccountId,
			snapshot_id: u32,
		},
		MainAccountRegistered {
			main: T::AccountId,
			proxy: T::AccountId,
		},
		TradingPairRegistered {
			base: AssetId,
			quote: AssetId,
		},
		TradingPairUpdated {
			base: AssetId,
			quote: AssetId,
		},
		DepositSuccessful {
			user: T::AccountId,
			asset: AssetId,
			amount: BalanceOf<T>,
		},
		ShutdownTradingPair {
			pair: TradingPairConfig,
		},
		OpenTradingPair {
			pair: TradingPairConfig,
		},
		EnclaveRegistered(T::AccountId),
		EnclaveAllowlisted(T::AccountId),
		EnclaveCleanup(Vec<T::AccountId>),
		TradingPairIsNotOperational,
		WithdrawalClaimed {
			main: T::AccountId,
			withdrawals: Vec<Withdrawal<T::AccountId>>,
		},
		NewProxyAdded {
			main: T::AccountId,
			proxy: T::AccountId,
		},
		ProxyRemoved {
			main: T::AccountId,
			proxy: T::AccountId,
		},
		/// TokenAllowlisted
		TokenAllowlisted(AssetId),
		/// AllowlistedTokenRemoved
		AllowlistedTokenRemoved(AssetId),
		/// Withdrawal failed
		WithdrawalFailed(Withdrawal<T::AccountId>),
		/// Exchange state has been updated
		ExchangeStateUpdated(bool),
		/// Withdraw Assets from Orderbook
		WithdrawFromOrderbook(T::AccountId, AssetId, BalanceOf<T>),
	}

	///Allowlisted tokens
	#[pallet::storage]
	#[pallet::getter(fn get_allowlisted_token)]
	pub(super) type AllowlistedToken<T: Config> =
		StorageValue<_, BoundedBTreeSet<AssetId, AllowlistedTokenLimit>, ValueQuery>;

	// A map that has enumerable entries.
	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub(super) type Accounts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		AccountInfo<T::AccountId, ProxyLimit>,
		OptionQuery,
	>;

	// Trading pairs registered as Base, Quote => TradingPairInfo
	#[pallet::storage]
	#[pallet::getter(fn trading_pairs)]
	pub(super) type TradingPairs<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AssetId,
		Blake2_128Concat,
		AssetId,
		TradingPairConfig,
		OptionQuery,
	>;

	// Unprocessed Snapshots storage ( snapshot id, summary_hash ) => SnapshotSummary
	#[pallet::storage]
	#[pallet::getter(fn unprocessed_snapshots)]
	pub(super) type UnprocessedSnapshots<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u64, Identity, H256, SnapshotSummary, OptionQuery>;

	// Snapshots Storage
	#[pallet::storage]
	#[pallet::getter(fn snapshots)]
	pub(super) type Snapshots<T: Config> =
		StorageMap<_, Blake2_128Concat, u64, SnapshotSummary, ValueQuery>;

	// Snapshots Nonce
	#[pallet::storage]
	#[pallet::getter(fn snapshot_nonce)]
	pub(super) type SnapshotNonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	// Snapshot will be produced after snapshot interval block
	#[pallet::storage]
	#[pallet::getter(fn snapshot_interval_block)]
	pub(super) type SnapshotIntervalBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	// Snapshot will be produced after reaching pending withdrawals limit
	#[pallet::storage]
	#[pallet::getter(fn pending_withdrawals_allowed_per_snapshot)]
	pub(super) type PendingWithdrawalsAllowedPerSnapshot<T: Config> =
		StorageValue<_, u64, ValueQuery>;

	// Exchange Operation State
	#[pallet::storage]
	#[pallet::getter(fn orderbook_operational_state)]
	pub(super) type ExchangeState<T: Config> = StorageValue<_, bool, ValueQuery>;

	// Unprocessed Snapshot from Previous set
	#[pallet::storage]
	#[pallet::getter(fn pending_snapshot_from_prev_set)]
	pub(super) type PendingSnapshotFromPreviousSet<T: Config> = StorageValue<_, u64, OptionQuery>;

	// Fees collected
	#[pallet::storage]
	#[pallet::getter(fn fees_collected)]
	pub(super) type FeesCollected<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, BoundedVec<Fees, AssetsLimit>, ValueQuery>;

	// Withdrawals mapped by their trading pairs and snapshot numbers
	#[pallet::storage]
	#[pallet::getter(fn withdrawals)]
	pub(super) type Withdrawals<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, WithdrawalsMap<T>, ValueQuery>;

	// Queue for enclave ingress messages
	#[pallet::storage]
	#[pallet::getter(fn ingress_messages)]
	pub(super) type IngressMessages<T: Config> = StorageValue<
		_,
		Vec<polkadex_primitives::ingress::IngressMessages<T::AccountId>>,
		ValueQuery,
	>;

	// Queue for onchain events
	#[pallet::storage]
	#[pallet::getter(fn onchain_events)]
	pub(super) type OnChainEvents<T: Config> = StorageValue<
		_,
		BoundedVec<polkadex_primitives::ocex::OnChainEvents<T::AccountId>, OnChainEventsLimit>,
		ValueQuery,
	>;

	// Total Assets present in orderbook
	#[pallet::storage]
	#[pallet::getter(fn total_assets)]
	pub(super) type TotalAssets<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId, Decimal, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_authorities)]
	pub(super) type Authorities<T: Config> =
		StorageValue<_, BoundedVec<AuthorityId, OnChainEventsLimit>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_next_authorities)]
	pub(super) type NextAuthorities<T: Config> =
		StorageValue<_, BoundedVec<AuthorityId, OnChainEventsLimit>, ValueQuery>;
}

// The main implementation block for the pallet. Functions here fall into three broad
// categories:
// - Public interface. These are functions that are `pub` and generally fall into inspector
// functions that do not write to storage and operation functions that do.
// - Private functions. These are your usual private utilities unavailable to other pallets.
impl<T: Config + frame_system::offchain::SendTransactionTypes<Call<T>>> Pallet<T> {
	pub fn validator_set() -> ValidatorSet<AuthorityId> {
		ValidatorSet { validators: <Authorities<T>>::get().into_inner() }
	}

	pub fn get_ingress_messages() -> Vec<polkadex_primitives::ingress::IngressMessages<T::AccountId>>
	{
		<IngressMessages<T>>::get()
	}

	pub fn submit_snapshot_api(summary: SnapshotSummary) -> Result<(), ()> {
		let call = Call::<T>::submit_snapshot { summary };
		SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
	}

	pub fn get_latest_snapshot() -> SnapshotSummary {
		let last_nonce = <SnapshotNonce<T>>::get();
		<Snapshots<T>>::get(last_nonce)
	}

	pub fn get_snapshot_by_id(nonce: u64) -> Option<SnapshotSummary> {
		let summary = <Snapshots<T>>::get(nonce);

		if summary == SnapshotSummary::default() {
			None
		} else {
			Some(summary)
		}
	}

	pub fn pending_snapshot() -> Option<u64> {
		<PendingSnapshotFromPreviousSet<T>>::get()
	}

	// Returns all main accounts and corresponding proxies for it at this point in time
	pub fn get_all_accounts_and_proxies() -> Vec<(T::AccountId, Vec<T::AccountId>)> {
		<Accounts<T>>::iter()
			.map(|(main, info)| (main, info.proxies.to_vec()))
			.collect::<Vec<(T::AccountId, Vec<T::AccountId>)>>()
	}

	// Returns allowlisted asset id's
	pub fn get_allowlisted_assets() -> Vec<AssetId> {
		<AllowlistedToken<T>>::get()
			.iter()
			.map(|asset_id| (asset_id.clone()))
			.collect::<Vec<AssetId>>()
	}

	pub fn get_snapshot_generation_intervals() -> (u64, T::BlockNumber) {
		let pending_withdrawals_interval = <PendingWithdrawalsAllowedPerSnapshot<T>>::get();
		let block_interval = <SnapshotIntervalBlock<T>>::get();
		(pending_withdrawals_interval, block_interval)
	}

	/// Returns the AccountId to hold user funds, note this account has no private keys and
	/// can accessed using on-chain logic.
	fn get_pallet_account() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	fn transfer_asset(
		payer: &T::AccountId,
		payee: &T::AccountId,
		amount: BalanceOf<T>,
		asset: AssetId,
	) -> DispatchResult {
		match asset {
			AssetId::polkadex => {
				T::NativeCurrency::transfer(
					payer,
					payee,
					amount.unique_saturated_into(),
					ExistenceRequirement::KeepAlive,
				)?;
			},
			AssetId::asset(id) => {
				T::OtherAssets::teleport(id, payer, payee, amount.unique_saturated_into())?;
			},
		}
		Ok(())
	}
}

impl<T: Config> sp_application_crypto::BoundToRuntimeAppPublic for Pallet<T> {
	type Public = orderbook_primitives::crypto::AuthorityId;
}

impl<T: Config> OneSessionHandler<T::AccountId> for Pallet<T> {
	type Key = orderbook_primitives::crypto::AuthorityId;

	fn on_genesis_session<'a, I: 'a>(authorities: I)
	where
		I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
	{
		let authorities = authorities.map(|(_, k)| k).collect::<Vec<_>>();
		if let Ok(bounded_authorities) = BoundedVec::try_from(authorities) {
			<Authorities<T>>::put(bounded_authorities);
		};
	}

	fn on_new_session<'a, I: 'a>(changed: bool, authorities: I, queued_authorities: I)
	where
		I: Iterator<Item = (&'a T::AccountId, Self::Key)>,
	{
		let next_authorities = authorities.map(|(_, k)| k).collect::<Vec<_>>();
		let next_queued_authorities = queued_authorities.map(|(_, k)| k).collect::<Vec<_>>();
		if let (Ok(next_bounded_authorities), Ok(next_bounded_queued_authorities)) =
			(BoundedVec::try_from(next_authorities), BoundedVec::try_from(next_queued_authorities))
		{
			if next_bounded_authorities != next_bounded_queued_authorities {
				<NextAuthorities<T>>::put(next_bounded_queued_authorities);
			}
			if changed {
				<Authorities<T>>::put(next_bounded_authorities);
				// Check if there is a pending snapshot from outgoing authority set
				let next_snapshot_id = <SnapshotNonce<T>>::get().saturating_add(1);
				let unprocessed_snapshots_from_outgoing_set =
					<UnprocessedSnapshots<T>>::iter_prefix(next_snapshot_id)
						.collect::<Vec<(H256, SnapshotSummary)>>();
				if !unprocessed_snapshots_from_outgoing_set.len().is_zero() {
					// if yes, signal the new validators to process it again.
					<PendingSnapshotFromPreviousSet<T>>::put(next_snapshot_id);
				}
			}
		}
	}

	fn on_disabled(_i: u32) {}
}
