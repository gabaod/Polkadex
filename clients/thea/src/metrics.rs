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

//! Thea Prometheus metrics definition.

use prometheus::{register, Counter, Gauge, PrometheusError, Registry, U64};

/// Thea metrics exposed through Prometheus.
pub struct Metrics {
	/// Last processed state id.
	pub thea_state_id: Gauge<U64>,
	/// Total number of ob messages sent by this node.
	pub thea_messages_sent: Counter<U64>,
	/// Total number of thea messages received by this node.
	pub thea_messages_recv: Counter<U64>,
	/// Total data sent out by thea worker.
	pub thea_data_sent: Gauge<U64>,
	/// Total data recv by thea worker.
	pub thea_data_recv: Gauge<U64>,
}

impl Metrics {
	/// Registers all predefined metrics collectors in registry.
	///
	/// # Parameters
	///
	/// * `registry`: Registry reference to register collectors in.
	pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
		Ok(Self {
			thea_state_id: register(
				Gauge::new("polkadex_thea_state_id", "Last processed state id by Thea")?,
				registry,
			)?,
			thea_messages_sent: register(
				Counter::new(
					"polkadex_thea_messages_sent",
					"Number of messages sent by this node",
				)?,
				registry,
			)?,
			thea_messages_recv: register(
				Counter::new(
					"polkadex_thea_messages_recv",
					"Number of messages received by this node",
				)?,
				registry,
			)?,
			thea_data_sent: register(
				Gauge::new("polkadex_thea_data_sent", "Total Data sent by Thea worker")?,
				registry,
			)?,
			thea_data_recv: register(
				Gauge::new("polkadex_thea_data_recv", "Total Data received by Thea worker")?,
				registry,
			)?,
		})
	}
}

// Note: we use the `format` macro to convert an expr into a `u64`. This will fail,
// if expr does not derive `Display`.
#[macro_export]
macro_rules! metric_set {
	($self:ident, $m:ident, $v:expr) => {{
		let val: u64 = format!("{}", $v).parse().unwrap();

		if let Some(metrics) = $self.metrics.as_ref() {
			metrics.$m.set(val);
		}
	}};
}

#[macro_export]
macro_rules! metric_inc {
	($self:ident, $m:ident) => {{
		if let Some(metrics) = $self.metrics.as_ref() {
			metrics.$m.inc();
		}
	}};
}

#[cfg(test)]
#[macro_export]
macro_rules! metric_get {
	($self:ident, $m:ident) => {{
		$self.metrics.as_ref().map(|metrics| metrics.$m.clone())
	}};
}

#[macro_export]
macro_rules! metric_add {
	($self:ident, $m:ident, $v:expr) => {{
		let val: u64 = format!("{}", $v).parse().unwrap();

		if let Some(metrics) = $self.metrics.as_ref() {
			metrics.$m.add(val);
		}
	}};
}
