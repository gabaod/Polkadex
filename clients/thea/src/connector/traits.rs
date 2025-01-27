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

//! This module contains foreign connector abstraction which should be implemented to be able to
//! connect to multiple chains.
//!
//! This module contains as well the concrete implementation of a foreign connector abstraction,
//! which used in development/testing environments as a stub.

use crate::{error::Error, types::GossipMessage};
use async_trait::async_trait;
use std::time::Duration;
use thea_primitives::types::Message;

/// Abstraction which should be implemented to be able to connect to multiple chains.
#[async_trait]
pub trait ForeignConnector: Send + Sync {
	/// Block duration.
	fn block_duration(&self) -> Duration;
	/// Initialize the connection to native blockchain.
	async fn connect(url: String) -> Result<Self, Error>
	where
		Self: Sized;
	/// Read all interested events based on the last processed nonce of that network on Polkadex.
	async fn read_events(&self, last_processed_nonce: u64) -> Result<Option<Message>, Error>;
	/// Sends transaction to blockchain, if failed, retry every second until its successful.
	async fn send_transaction(&self, message: GossipMessage) -> Result<(), Error>;
	/// Checks if the given message is valid or not based on our local node.
	async fn check_message(&self, message: &Message) -> Result<bool, Error>;
	/// Returns the last processed nonce from native chain.
	async fn last_processed_nonce_from_native(&self) -> Result<u64, Error>;
	/// Check if the foreign chain is initialized with thea validators.
	async fn check_thea_authority_initialization(&self) -> Result<bool, Error>;
}

/// ForeignConnector that does nothing, mainly used for starting node in development mode
/// for just testing runtime as a stub.
pub struct NoOpConnector;

#[async_trait]
impl ForeignConnector for NoOpConnector {
	fn block_duration(&self) -> Duration {
		Duration::from_secs(60)
	}

	async fn connect(_: String) -> Result<Self, Error>
	where
		Self: Sized,
	{
		Ok(NoOpConnector)
	}

	async fn read_events(&self, _: u64) -> Result<Option<Message>, Error> {
		Ok(None)
	}

	async fn send_transaction(&self, _: GossipMessage) -> Result<(), Error> {
		Ok(())
	}

	async fn check_message(&self, _: &Message) -> Result<bool, Error> {
		Ok(false)
	}

	async fn last_processed_nonce_from_native(&self) -> Result<u64, Error> {
		Ok(0)
	}

	async fn check_thea_authority_initialization(&self) -> Result<bool, Error> {
		Ok(false)
	}
}
