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

//! Definition of types used inside Thea client.

use bls_primitives::Signature;
use parity_scale_codec::{Decode, Encode};
use thea_primitives::types::Message;

/// Representation of the gossip message structure.
#[derive(Encode, Decode, Clone, Debug)]
pub struct GossipMessage {
	/// Payload of the gossip message.
	pub(crate) payload: Message,
	/// Bitmap generated from active validators.
	pub(crate) bitmap: Vec<u128>,
	/// Message aggregated signature.
	pub(crate) aggregate_signature: Signature,
}
