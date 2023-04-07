mod id;
mod entry;
mod persistence;

use std::collections::HashMap;

use paste::paste;

pub use id::FeedsId;
pub use entry::FeedsEntry;

pub struct Feeds {
    entries: HashMap<FeedsId, FeedsEntry>,
}

crate::api_impl!(feeds, FeedsId, FeedsEntry);
crate::module_impl!(Feeds, FeedsId, FeedsEntry);
