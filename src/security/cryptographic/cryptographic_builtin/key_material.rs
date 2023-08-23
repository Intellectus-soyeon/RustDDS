use byteorder::BigEndian;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{
  security::{
    SecurityError, SecurityResult, 
  },
  security_error,
  serialization::cdr_serializer::to_bytes,
  CdrDeserializer,
};
use super::{
  CryptoToken, CryptoTransformKeyId, CryptoTransformKind,
  BuiltinCryptoTransformationKind, BuiltinCryptoToken};

use super::builtin_key::*;

/// KeyMaterial_AES_GCM_GMAC type from section 9.5.2.1.1 of the Security
/// specification (v. 1.1)
#[allow(non_camel_case_types)] // We use the name from the spec
#[derive(Clone)]
pub(crate) struct KeyMaterial_AES_GCM_GMAC {
  pub transformation_kind: BuiltinCryptoTransformationKind,
  pub master_salt: Vec<u8>,
  pub sender_key_id: CryptoTransformKeyId,
  pub master_sender_key: BuiltinKey,
  pub receiver_specific_key_id: CryptoTransformKeyId,
  pub master_receiver_specific_key: BuiltinKey,
}

// Conversions from and into Bytes
impl TryFrom<Bytes> for KeyMaterial_AES_GCM_GMAC {
  type Error = SecurityError;
  fn try_from(value: Bytes) -> Result<Self, Self::Error> {
    // Deserialize CDR-formatted key material
    Serializable_KeyMaterial_AES_GCM_GMAC::deserialize(&mut CdrDeserializer::<
      BigEndian, /* TODO: What's the point of this constructor if we need to specify the byte
                  * order anyway */
    >::new_big_endian(value.as_ref()))
    .map_err(
      // Map deserialization error to SecurityError
      |e| Self::Error {
        msg: format!("Error deserializing KeyMaterial_AES_GCM_GMAC: {}", e),
      },
    )
    .and_then(KeyMaterial_AES_GCM_GMAC::try_from)
  }
}
impl TryFrom<KeyMaterial_AES_GCM_GMAC> for Bytes {
  type Error = SecurityError;
  fn try_from(key_material: KeyMaterial_AES_GCM_GMAC) -> Result<Self, Self::Error> {
    // Convert the key material to the serializable structure
    let serializable_key_material = Serializable_KeyMaterial_AES_GCM_GMAC::from(key_material);
    // Serialize
    to_bytes::<Serializable_KeyMaterial_AES_GCM_GMAC, BigEndian>(&serializable_key_material)
      .map(Bytes::from)
      .map_err(|e| Self::Error {
        msg: format!("Error serializing KeyMaterial_AES_GCM_GMAC: {}", e),
      })
  }
}

// Conversions from and into CryptoToken
impl TryFrom<CryptoToken> for KeyMaterial_AES_GCM_GMAC {
  type Error = SecurityError;
  fn try_from(token: CryptoToken) -> Result<Self, Self::Error> {
    BuiltinCryptoToken::try_from(token).map(KeyMaterial_AES_GCM_GMAC::from)
  }
}
impl TryFrom<KeyMaterial_AES_GCM_GMAC> for CryptoToken {
  type Error = SecurityError;
  fn try_from(key_material: KeyMaterial_AES_GCM_GMAC) -> Result<Self, Self::Error> {
    BuiltinCryptoToken::from(key_material).try_into()
  }
}

/// We need to refer to a sequence of key material structures for example in
/// register_local_datawriter. Usually the sequence has one key material, but it
/// can have two if different key materials is used for submessage and payload
#[allow(non_camel_case_types)] // We use the name from the spec
#[derive(Clone)]
pub(crate) enum KeyMaterial_AES_GCM_GMAC_seq {
  One(KeyMaterial_AES_GCM_GMAC),
  Two(KeyMaterial_AES_GCM_GMAC, KeyMaterial_AES_GCM_GMAC),
}

impl KeyMaterial_AES_GCM_GMAC_seq {
  pub fn key_material(&self) -> &KeyMaterial_AES_GCM_GMAC {
    match self {
      Self::One(key_material) => key_material,
      Self::Two(key_material, _) => key_material,
    }
  }

  pub fn payload_key_material(&self) -> &KeyMaterial_AES_GCM_GMAC {
    match self {
      Self::One(key_material) => key_material,
      Self::Two(_, payload_key_material) => payload_key_material,
    }
  }

  pub fn modify_key_material<F>(self, f: F) -> KeyMaterial_AES_GCM_GMAC_seq
  where
    F: FnOnce(KeyMaterial_AES_GCM_GMAC) -> KeyMaterial_AES_GCM_GMAC,
  {
    match self {
      Self::One(key_material) => Self::One(f(key_material)),
      Self::Two(key_material, payload_key_material) => {
        Self::Two(f(key_material), payload_key_material)
      }
    }
  }

  pub fn add_master_receiver_specific_key(
    self,
    receiver_specific_key_id: CryptoTransformKeyId,
    master_receiver_specific_key: BuiltinKey,
  ) -> KeyMaterial_AES_GCM_GMAC_seq {
    self.modify_key_material(
      |KeyMaterial_AES_GCM_GMAC {
         transformation_kind,
         master_salt,
         master_sender_key,
         sender_key_id,
         ..
       }| KeyMaterial_AES_GCM_GMAC {
        transformation_kind,
        master_salt,
        master_sender_key,
        sender_key_id,
        receiver_specific_key_id,
        master_receiver_specific_key,
      },
    )
  }
}

impl TryFrom<Vec<KeyMaterial_AES_GCM_GMAC>> for KeyMaterial_AES_GCM_GMAC_seq {
  type Error = SecurityError;
  fn try_from(value: Vec<KeyMaterial_AES_GCM_GMAC>) -> Result<Self, Self::Error> {
    match value.as_slice() {
      [key_material] => Ok(KeyMaterial_AES_GCM_GMAC_seq::One(key_material.clone())),
      [key_material, payload_key_material] => Ok(KeyMaterial_AES_GCM_GMAC_seq::Two(
        key_material.clone(),
        payload_key_material.clone(),
      )),
      [] => Ok(KeyMaterial_AES_GCM_GMAC_seq::One(
        KeyMaterial_AES_GCM_GMAC {
          transformation_kind: BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_NONE,
          master_salt: Vec::new(),
          sender_key_id: 0,
          master_sender_key: BuiltinKey::ZERO,
          receiver_specific_key_id: 0,
          master_receiver_specific_key: BuiltinKey::ZERO,
        },
      )),
      _ => Err(security_error!(
        "Expected 1 or 2 key materials in KeyMaterial_AES_GCM_GMAC_seq, received {}",
        value.len()
      )),
    }
  }
}
impl From<KeyMaterial_AES_GCM_GMAC_seq> for Vec<KeyMaterial_AES_GCM_GMAC> {
  fn from(key_materials: KeyMaterial_AES_GCM_GMAC_seq) -> Self {
    match key_materials {
      KeyMaterial_AES_GCM_GMAC_seq::One(key_material) => vec![key_material],
      KeyMaterial_AES_GCM_GMAC_seq::Two(key_material, payload_key_material) => {
        vec![key_material, payload_key_material]
      }
    }
  }
}

// Conversions from and into Bytes for KeyMaterial_AES_GCM_GMAC_seq
impl TryFrom<Bytes> for KeyMaterial_AES_GCM_GMAC_seq {
  type Error = SecurityError;
  fn try_from(value: Bytes) -> Result<Self, Self::Error> {
    // Deserialize CDR-formatted key material
    let serializable_key_materials =
      Vec::<Serializable_KeyMaterial_AES_GCM_GMAC>::deserialize(&mut CdrDeserializer::<
        BigEndian, /* TODO: What's the point of this constructor if we need to specify the byte
                    * order anyway */
      >::new_big_endian(
        value.as_ref()
      ))
      .map_err(
        // Map deserialization error to SecurityError
        |e| Self::Error {
          msg: format!("Error deserializing Vec<KeyMaterial_AES_GCM_GMAC>: {}", e),
        },
      )?;

    serializable_key_materials
      // Map transformation_kind to builtin for each keymat
      .iter()
      .map(|serializable_key_material| {
        KeyMaterial_AES_GCM_GMAC::try_from(serializable_key_material.clone())
      })
      // Convert to Vec and dig out the Result
      .collect::<Result<Vec<KeyMaterial_AES_GCM_GMAC>, Self::Error>>()
      // Convert the Vec
      .and_then(KeyMaterial_AES_GCM_GMAC_seq::try_from)
  }
}

impl TryFrom<KeyMaterial_AES_GCM_GMAC_seq> for Bytes {
  type Error = SecurityError;
  fn try_from(key_materials: KeyMaterial_AES_GCM_GMAC_seq) -> Result<Self, Self::Error> {
    // Convert the key material to the serializable structure
    let serializable_key_materials = Vec::from(key_materials)
      .iter()
      .map(|key_material| Serializable_KeyMaterial_AES_GCM_GMAC::from(key_material.clone()))
      .collect();

    // Serialize
    to_bytes::<Vec<Serializable_KeyMaterial_AES_GCM_GMAC>, BigEndian>(&serializable_key_materials)
      .map(Bytes::from)
      .map_err(|e| Self::Error {
        msg: format!("Error serializing KeyMaterial_AES_GCM_GMAC_seq: {}", e),
      })
  }
}

impl KeyMaterial_AES_GCM_GMAC {
  /// Checks that the key material matches the given common key material and
  /// returns the receiver-specific material
  pub fn receiver_key_material_for(
    &self,
    KeyMaterial_AES_GCM_GMAC {
      transformation_kind,
      master_salt,
      sender_key_id,
      master_sender_key,
      ..
    }: &KeyMaterial_AES_GCM_GMAC,
  ) -> SecurityResult<ReceiverKeyMaterial> {
    if !self.sender_key_id.eq(sender_key_id) {
      Err(security_error!(
        "The receiver-specific key material has a wrong sender_key_id: expected {:?}, received \
         {:?}.",
        sender_key_id,
        self.sender_key_id
      ))
    } else if !self.transformation_kind.eq(transformation_kind) {
      Err(security_error!(
        "The receiver-specific key material has a wrong transformation_kind: expected {:?}, \
         received {:?}.",
        transformation_kind,
        self.transformation_kind
      ))
    } else if !self.master_sender_key.eq(master_sender_key) {
      Err(security_error!(
        "The receiver-specific key has a wrong master_sender_key: expected {:?}, received {:?}.",
        master_sender_key,
        self.master_sender_key
      ))
    } else if !self.master_salt.eq(master_salt) {
      Err(security_error!(
        "The receiver-specific key has a wrong master_salt: expected {:?}, received {:?}.",
        master_salt,
        self.master_salt
      ))
    } else {
      Ok(ReceiverKeyMaterial {
        receiver_specific_key_id: self.receiver_specific_key_id,
        master_receiver_specific_key: self.master_receiver_specific_key.clone(),
      })
    }
  }
}

pub(crate) struct ReceiverKeyMaterial {
  pub receiver_specific_key_id: CryptoTransformKeyId,
  pub master_receiver_specific_key: BuiltinKey,
}

// Conversions from and into Vec<CryptoToken> for KeyMaterial_AES_GCM_GMAC_seq
impl TryFrom<Vec<CryptoToken>> for KeyMaterial_AES_GCM_GMAC_seq {
  type Error = SecurityError;
  fn try_from(tokens: Vec<CryptoToken>) -> Result<Self, Self::Error> {
    tokens
      .iter()
      .map(|token| KeyMaterial_AES_GCM_GMAC::try_from(token.clone()))
      .collect::<Result<Vec<KeyMaterial_AES_GCM_GMAC>, Self::Error>>()
      // Convert the Vec
      .and_then(KeyMaterial_AES_GCM_GMAC_seq::try_from)
  }
}
impl TryFrom<KeyMaterial_AES_GCM_GMAC_seq> for Vec<CryptoToken> {
  type Error = SecurityError;
  fn try_from(key_materials: KeyMaterial_AES_GCM_GMAC_seq) -> Result<Self, Self::Error> {
    Vec::from(key_materials)
      .iter()
      .map(|key_material| CryptoToken::try_from(key_material.clone()))
      .collect()
  }
}
//For (de)serialization
// See definition in DDS Security spec v1.1
// "9.5.2.1.1 KeyMaterial_AES_GCM_GMAC structure"
#[allow(non_camel_case_types)] // We use the name from the spec
#[derive(Deserialize, Serialize, PartialEq, Clone)]
struct Serializable_KeyMaterial_AES_GCM_GMAC {
  transformation_kind: CryptoTransformKind,
  master_salt: Vec<u8>, // sequence<octet, 32>
  sender_key_id: CryptoTransformKeyId,
  master_sender_key: Vec<u8>, // sequence<octet, 32>
  receiver_specific_key_id: CryptoTransformKeyId,
  master_receiver_specific_key: Vec<u8>, // sequence<octet, 32>
}

// The `sequence<octet, 32>` IDL type in the spec means variable-length
// sequence. Vec<u8> is encoding-compatible, as long as we limit the length to
// 32.

impl TryFrom<Serializable_KeyMaterial_AES_GCM_GMAC> for KeyMaterial_AES_GCM_GMAC {
  type Error = SecurityError;
  fn try_from(
    Serializable_KeyMaterial_AES_GCM_GMAC {
      transformation_kind,
      master_salt,
      sender_key_id,
      master_sender_key,
      receiver_specific_key_id,
      master_receiver_specific_key,
    }: Serializable_KeyMaterial_AES_GCM_GMAC,
  ) -> Result<Self, Self::Error> {
    // Map generic transformation_kind to builtin
    let transformation_kind = BuiltinCryptoTransformationKind::try_from(transformation_kind)?;

    let key_len = KeyLength::try_from(transformation_kind)?;

    Ok(Self {
      transformation_kind,
      master_salt,
      sender_key_id,
      master_sender_key: BuiltinKey::from_bytes(key_len, &master_sender_key)?,
      receiver_specific_key_id,
      master_receiver_specific_key: BuiltinKey::from_bytes(key_len, &master_receiver_specific_key)?,
    })
  }
}

impl From<KeyMaterial_AES_GCM_GMAC> for Serializable_KeyMaterial_AES_GCM_GMAC {
  fn from(
    KeyMaterial_AES_GCM_GMAC {
      transformation_kind,
      master_salt,
      sender_key_id,
      master_sender_key,
      receiver_specific_key_id,
      master_receiver_specific_key,
    }: KeyMaterial_AES_GCM_GMAC,
  ) -> Self {
    Serializable_KeyMaterial_AES_GCM_GMAC {
      transformation_kind: transformation_kind.into(),
      master_salt,
      sender_key_id,
      master_sender_key: master_sender_key.as_bytes().into(),
      receiver_specific_key_id,
      master_receiver_specific_key: master_receiver_specific_key.as_bytes().into(),
    }
  }
}
