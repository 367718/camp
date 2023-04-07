mod id;
mod entry;
mod persistence;

use std::collections::HashMap;

use paste::paste;

pub use id::KindsId;
pub use entry::KindsEntry;

pub struct Kinds {
    entries: HashMap<KindsId, KindsEntry>,
}

crate::api_impl!(kinds, KindsId, KindsEntry);
crate::module_impl!(Kinds, KindsId, KindsEntry);
