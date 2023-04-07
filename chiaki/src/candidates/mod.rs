mod id;
mod entry;
mod persistence;

use std::collections::HashMap;

use paste::paste;

use crate::{ Series, SeriesId };

pub use id::CandidatesId;
pub use entry::{ CandidatesEntry, CandidatesCurrent };

pub struct Candidates {
    entries: HashMap<CandidatesId, CandidatesEntry>,
}

crate::api_impl!(candidates, CandidatesId, CandidatesEntry, series: &Series);
crate::module_impl!(Candidates, CandidatesId, CandidatesEntry, series: &Series);
