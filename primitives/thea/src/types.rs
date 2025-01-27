// This file is part of Polkadex.
//
// Copyright (c) 2023 Polkadex oü.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Definition of types used for `Thea` related operations.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::Percent;
use sp_std::cmp::Ordering;
#[cfg(not(feature = "std"))]
use sp_std::vec::Vec;

use crate::{Network, ValidatorSetId};

/// Defines the message structure.
#[derive(Clone, Encode, Decode, TypeInfo, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Message {
	/// Block number.
	pub block_no: u64,
	/// Message nonce (e.g. identifier).
	pub nonce: u64,
	/// Payload of the message.
	pub data: Vec<u8>,
	/// Message originated from this network
	pub network: Network,
	/// Defines if authority was changed.
	pub is_key_change: bool,
	/// Validator set id at which this message was executed.
	pub validator_set_id: ValidatorSetId,
	/// Validators authorities set length.
	pub validator_set_len: u64,
}

impl Message {
	/// Calculates message validators threshold percentage.
	pub fn threshold(&self) -> u64 {
		const MAJORITY: u8 = 67;
		let p = Percent::from_percent(MAJORITY);
		p * self.validator_set_len
	}
}

/// Defines structure of the deposit.
///
/// Deposit is relative to the "solochain".
#[derive(Encode, Decode, Clone, TypeInfo, PartialEq, Debug)]
pub struct Deposit<AccountId> {
	/// Identifier of the deposit.
	pub id: Vec<u8>, // Unique identifier
	/// Receiver of the deposit.
	pub recipient: AccountId,
	/// Asset identifier.
	pub asset_id: u128,
	/// Amount of the deposit.
	pub amount: u128,
	/// Extra data.
	pub extra: Vec<u8>,
}

impl<AccountId> Deposit<AccountId> {
	pub fn amount_in_native_decimals(&self, metadata: AssetMetadata) -> u128 {
		metadata.convert_to_native_decimals(self.amount)
	}

	pub fn amount_in_foreign_decimals(&self, metadata: AssetMetadata) -> u128 {
		metadata.convert_from_native_decimals(self.amount)
	}
}

/// Defines the structure of the withdraw.
///
/// Withdraw is relative to solochain
#[derive(Encode, Decode, Clone, TypeInfo, PartialEq, Debug)]
pub struct Withdraw {
	/// Identifier of the withdrawal.
	pub id: Vec<u8>,
	// Unique identifier
	/// Asset identifier.
	pub asset_id: u128,
	/// Amount of the withdrawal.
	pub amount: u128,
	/// Receiver of the withdrawal.
	pub destination: Vec<u8>,
	/// Defines if withdraw operation is blocked.
	pub is_blocked: bool,
	/// Extra data.
	pub extra: Vec<u8>,
}

/// Metadata of asset's decimals
#[derive(Encode, Decode, Clone, TypeInfo, PartialEq, Debug, Copy)]
pub struct AssetMetadata {
	decimal: u8,
}

impl AssetMetadata {
	pub fn new(decimal: u8) -> Option<AssetMetadata> {
		if decimal < 1 {
			return None
		}
		Some(AssetMetadata { decimal })
	}

	/// Convert the foreign asset amount to native decimal configuration
	pub fn convert_to_native_decimals(&self, amount: u128) -> u128 {
		let diff = 12 - self.decimal as i8;
		match diff.cmp(&0) {
			Ordering::Less => {
				// casting should not fail as diff*-1 is positive
				amount.saturating_div(10u128.pow((-diff) as u32))
			},
			Ordering::Equal => amount,
			Ordering::Greater => amount.saturating_mul(10u128.pow(diff as u32)),
		}
	}

	/// Convert the foreign asset amount from native decimal configuration
	pub fn convert_from_native_decimals(&self, amount: u128) -> u128 {
		let diff = 12 - self.decimal as i8;

		match diff.cmp(&0) {
			Ordering::Less => {
				// casting should not fail as diff*-1 is positive
				amount.saturating_mul(10u128.pow((-diff) as u32))
			},
			Ordering::Equal => amount,
			Ordering::Greater => amount.saturating_div(10u128.pow(diff as u32)),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::types::AssetMetadata;
	use polkadex_primitives::UNIT_BALANCE;

	#[test]
	pub fn test_decimal_conversion() {
		// Decimal is greater
		let greater = AssetMetadata::new(18).unwrap();
		assert_eq!(greater.convert_to_native_decimals(1000_000_000_000_000_000u128), UNIT_BALANCE);
		assert_eq!(
			greater.convert_from_native_decimals(UNIT_BALANCE),
			1000_000_000_000_000_000u128
		);
		assert_eq!(
			greater.convert_to_native_decimals(1234_567_891_234_567_890u128),
			1234_567_891_234u128
		);
		assert_eq!(
			greater.convert_from_native_decimals(1234_567_891_234u128),
			1234_567_891_234_000_000u128
		);

		// Decimal is same
		let same = AssetMetadata::new(12).unwrap();
		assert_eq!(same.convert_to_native_decimals(UNIT_BALANCE), UNIT_BALANCE);
		assert_eq!(same.convert_from_native_decimals(UNIT_BALANCE), UNIT_BALANCE);

		// Decimal is lesser
		let smaller = AssetMetadata::new(8).unwrap();
		assert_eq!(smaller.convert_to_native_decimals(100_000_000), UNIT_BALANCE);
		assert_eq!(smaller.convert_from_native_decimals(UNIT_BALANCE), 100_000_000);
		assert_eq!(smaller.convert_to_native_decimals(12_345_678u128), 123_456_780_000u128);
		assert_eq!(smaller.convert_from_native_decimals(123_456_789_123u128), 12_345_678u128);
	}
}
