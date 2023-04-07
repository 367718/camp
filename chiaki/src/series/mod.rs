mod id;
mod entry;
mod persistence;

use std::collections::HashMap;

use paste::paste;

use crate::{ Kinds, KindsId };

pub use id::SeriesId;
pub use entry::{ SeriesEntry, SeriesStatus, SeriesGood };

pub struct Series {
    entries: HashMap<SeriesId, SeriesEntry>,
}

crate::api_impl!(series, SeriesId, SeriesEntry, kinds: &Kinds);
crate::module_impl!(Series, SeriesId, SeriesEntry, kinds: &Kinds);
