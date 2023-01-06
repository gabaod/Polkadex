use crate::{pallet::Stakinglimits, session::Exposure, Config};

/// Algorithm to elect relayers from all candidates
pub fn elect_relayers<T: Config>(
	mut candidates: Vec<(T::AccountId, Exposure<T, T::AccountId>)>,
) -> Vec<(T::AccountId, Exposure<T, T::AccountId>)> {
	// If we don't have preffered number of relayers we take everyone
	let max_relayers = <Stakinglimits<T>>::get().max_relayers;
	if candidates.len() <= max_relayers as usize {
		return candidates
	}
	// Sort by the descending order of total stake if we have more candidates than MaxRelayers
	candidates.sort_unstable_by(|(_, ea), (_, eb)| eb.total.cmp(&ea.total));
	let _ = candidates.split_off(max_relayers as usize);
	// Take the top MaxRelayers relayers
	candidates
}
