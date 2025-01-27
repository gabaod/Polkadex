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

use crate::types::AccountAsset;
use parity_scale_codec::{Decode, Encode};
use polkadex_primitives::{AccountId, BlockNumber};
use rust_decimal::Decimal;
use serde_with::{json::JsonString, serde_as};
use std::collections::BTreeMap;

/// A struct representing the recovery state of an Order Book.
#[serde_as]
#[derive(Clone, Debug, Encode, Decode, Default, serde::Serialize, serde::Deserialize)]
pub struct ObRecoveryState {
	/// The snapshot ID of the order book recovery state.
	pub snapshot_id: u64,
	/// A `BTreeMap` that maps main account to a vector of proxy account.
	#[serde_as(as = "JsonString<Vec<(JsonString, _)>>")]
	pub account_ids: BTreeMap<AccountId, Vec<AccountId>>,
	/// A `BTreeMap` that maps `AccountAsset`s to `Decimal` balances.
	#[serde_as(as = "JsonString<Vec<(JsonString, _)>>")]
	pub balances: BTreeMap<AccountAsset, Decimal>,
	/// The last block number that was processed by validator.
	pub last_processed_block_number: BlockNumber,
	/// State change id
	pub state_change_id: u64,
	/// worker nonce
	pub worker_nonce: u64,
	/// State version
	pub state_version: u16,
}
