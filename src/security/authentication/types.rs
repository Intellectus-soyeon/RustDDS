use serde::{Deserialize, Serialize};

// ValidationOutcome is like ValidationResult_t in the the Security
// specification v.1.1 (section 8.3.2.11.1), but does not contain
// VALIDATION_FAILED. Failure is handled as an error in the result type
// ValidationResult
pub enum ValidationOutcome {
  Ok,
  PendingRetry,
  PendingHandshakeRequest,
  PendingHandshakeMessage,
  OkFinalMessage,
}

// TODO: IdentityToken: section 8.3.2.1 of the Security specification (v. 1.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityToken {}

impl IdentityToken {
  // Mock value used for development
  pub const MOCK: Self = Self {};
}

// TODO: IdentityStatusToken: section 8.3.2.2 of the Security specification (v.
// 1.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityStatusToken {}

impl IdentityStatusToken {
  // Mock value used for development
  pub const MOCK: Self = Self {};
}

// TODO: IdentityHandle: section 8.3.2.3 of the Security specification (v. 1.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityHandle {}

impl IdentityHandle {
  // Mock value used for development
  pub const MOCK: Self = Self {};
}

// TODO: HandshakeHandle: section 8.3.2.4 of the Security specification (v. 1.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeHandle {}

impl HandshakeHandle {
  // Mock value used for development
  pub const MOCK: Self = Self {};
}

// TODO: AuthRequestMessageToken: section 8.3.2.5 of the Security specification
// (v. 1.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthRequestMessageToken {}

impl AuthRequestMessageToken {
  // Mock value used for development
  pub const MOCK: Self = Self {};
}

// TODO: HandshakeMessageToken: section 8.3.2.6 of the Security specification
// (v. 1.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeMessageToken {}

impl HandshakeMessageToken {
  // Mock value used for development
  pub const MOCK: Self = Self {};
}

// TODO: AuthenticatedPeerCredentialToken: section 8.3.2.7 of the Security
// specification (v. 1.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticatedPeerCredentialToken {}

impl AuthenticatedPeerCredentialToken {
  // Mock value used for development
  pub const MOCK: Self = Self {};
}

// TODO: SharedSecretHandle: section 8.3.2.8 of the Security specification (v.
// 1.1)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedSecretHandle {}

impl SharedSecretHandle {
  // Mock value used for development
  pub const MOCK: Self = Self {};
}
