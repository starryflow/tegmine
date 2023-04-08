use base58::ToBase58;
use tegmine_common::prelude::*;
use uuid::Uuid;

/// ID Generator Note on overriding the ID Generator: The default ID generator uses UUID v4 as the
/// ID format. By overriding this class it is possible to use different scheme for ID generation.
/// However, this is not normal and should only be done after very careful consideration.
pub struct IdGenerator;

impl IdGenerator {
    pub fn generate() -> InlineStr {
        Uuid::new_v4().as_bytes().to_base58().into()
    }
}
