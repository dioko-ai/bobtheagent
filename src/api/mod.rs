pub mod capabilities;
pub mod contracts;
pub mod envelope;

pub use capabilities::{CAPABILITY_MATRIX, CapabilityDomain, CapabilityId, capability_definition};
pub use contracts::*;
pub use envelope::{
    ApiErrorCode, ApiErrorEnvelope, ApiResultEnvelope, RequestEnvelope, RequestMetadata,
    ResponseEnvelope,
};

#[cfg(test)]
#[path = "../../tests/unit/api_tests.rs"]
mod tests;
