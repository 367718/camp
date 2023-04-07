mod id;
mod entry;
mod persistence;

use std::collections::HashMap;

use paste::paste;

pub use id::FormatsId;
pub use entry::FormatsEntry;

pub struct Formats {
    entries: HashMap<FormatsId, FormatsEntry>,
}

crate::api_impl!(formats, FormatsId, FormatsEntry);
crate::module_impl!(Formats, FormatsId, FormatsEntry);
