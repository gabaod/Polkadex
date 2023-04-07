
//! Autogenerated weights for `thea`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-07, STEPS: `100`, REPEAT: 200, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `Ubuntu-2204-jammy-amd64-base`, CPU: `Intel(R) Core(TM) i7-7700 CPU @ 3.60GHz`
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// ./polkadex-node
// benchmark
// pallet
// --pallet
// thea
// --steps
// 100
// --repeat
// 200
// --extrinsic
// *
// --output
// thea_weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `thea`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> crate::WeightInfo for WeightInfo<T> {
	// Storage: Thea DepositNonce (r:1 w:1)
	// Storage: AssetHandler TheaAssets (r:1 w:0)
	// Storage: Thea RelayersBLSKeyVector (r:1 w:0)
	// Storage: Thea ApprovedDeposits (r:1 w:1)
	// Storage: Thea AccountWithPendingDeposits (r:1 w:1)
	fn approve_deposit() -> Weight {
		// Minimum execution time: 719_036 nanoseconds.
		Weight::from_ref_time(742_265_000)
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Thea ApprovedDeposits (r:1 w:1)
	// Storage: AssetHandler TheaAssets (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: Thea AccountWithPendingDeposits (r:1 w:1)
	/// The range of component `a` is `[0, 255]`.
	/// The range of component `m` is `[100, 4294967295]`.
	fn claim_deposit(_a: u32, _m: u32, ) -> Weight {
		// Minimum execution time: 657_113 nanoseconds.
		Weight::from_ref_time(672_285_652)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Thea WithdrawalNonces (r:1 w:0)
	// Storage: Thea RelayersBLSKeyVector (r:1 w:0)
	// Storage: Thea ReadyWithdrawls (r:1 w:0)
	fn batch_withdrawal_complete() -> Weight {
		// Minimum execution time: 728_318 nanoseconds.
		Weight::from_ref_time(733_666_000)
			.saturating_add(T::DbWeight::get().reads(3))
	}
	// Storage: AssetHandler TheaAssets (r:1 w:0)
	// Storage: Thea TheaKeyRotation (r:1 w:0)
	// Storage: Thea WithdrawalNonces (r:1 w:1)
	// Storage: Thea PendingWithdrawals (r:1 w:1)
	// Storage: Thea WithdrawalFees (r:1 w:0)
	// Storage: Assets Asset (r:1 w:1)
	// Storage: Assets Account (r:1 w:1)
	// Storage: Thea ReadyWithdrawls (r:0 w:1)
	fn withdraw() -> Weight {
		// Minimum execution time: 57_458 nanoseconds.
		Weight::from_ref_time(58_162_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: Thea WithdrawalFees (r:0 w:1)
	fn set_withdrawal_fee() -> Weight {
		// Minimum execution time: 14_538 nanoseconds.
		Weight::from_ref_time(14_961_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Thea ForeignChainAckTxns (r:1 w:1)
	// Storage: Thea RelayersBLSKeyVector (r:1 w:1)
	// Storage: Thea AuthorityListVector (r:1 w:1)
	// Storage: Thea QueuedRelayersBLSKeyVector (r:1 w:0)
	// Storage: Thea QueuedAuthorityListVector (r:1 w:0)
	// Storage: Thea QueuedTheaPublicKey (r:1 w:0)
	// Storage: Thea TheaSessionId (r:1 w:1)
	// Storage: Thea IngressMessages (r:1 w:1)
	// Storage: TheaStaking CurrentIndex (r:1 w:0)
	// Storage: TheaStaking EraRewardPoints (r:1 w:1)
	// Storage: Thea TheaPublicKey (r:0 w:1)
	// Storage: Thea TheaKeyRotation (r:0 w:1)
	fn thea_key_rotation_complete() -> Weight {
		// Minimum execution time: 747_886 nanoseconds.
		Weight::from_ref_time(752_808_000)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	// Storage: Thea TheaPublicKey (r:1 w:1)
	// Storage: Thea RelayersBLSKeyVector (r:1 w:0)
	// Storage: Thea AuthorityListVector (r:1 w:0)
	// Storage: Thea TheaSessionId (r:1 w:1)
	// Storage: TheaStaking CurrentIndex (r:1 w:0)
	// Storage: TheaStaking EraRewardPoints (r:1 w:1)
	// Storage: Thea TheaKeyRotation (r:0 w:1)
	fn set_thea_key_complete() -> Weight {
		// Minimum execution time: 738_466 nanoseconds.
		Weight::from_ref_time(742_210_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: Thea QueuedQueuedTheaPublicKey (r:1 w:1)
	// Storage: TheaStaking QueuedRelayers (r:1 w:0)
	fn thea_queued_queued_public_key() -> Weight {
		// Minimum execution time: 723_866 nanoseconds.
		Weight::from_ref_time(727_378_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Thea TheaPublicKey (r:1 w:0)
	// Storage: Thea QueuedTheaPublicKey (r:1 w:0)
	// Storage: Thea QueuedQueuedTheaPublicKey (r:1 w:0)
	// Storage: Thea QueuedAuthorityListVector (r:0 w:1)
	// Storage: Thea RelayersBLSKeyVector (r:0 w:1)
	// Storage: Thea QueuedRelayersBLSKeyVector (r:0 w:1)
	// Storage: Thea AuthorityListVector (r:0 w:1)
	// Storage: Thea TheaSessionId (r:0 w:1)
	fn thea_relayers_reset_rotation() -> Weight {
		// Minimum execution time: 13_312 nanoseconds.
		Weight::from_ref_time(13_792_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(5))
	}
}
