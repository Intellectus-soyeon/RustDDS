use byteorder::BigEndian;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{
  messages::submessages::submessage_elements::{
    crypto_content::CryptoContent, crypto_footer::CryptoFooter, crypto_header::CryptoHeader,
  },
  security::{BinaryProperty, DataHolder, SecurityError},
  serialization::cdr_serializer::to_bytes,
  CdrDeserializer,
};
use super::types::{
  CryptoToken, CryptoTransformIdentifier, CryptoTransformKeyId, CryptoTransformKind,
};

const CRYPTO_TOKEN_CLASS_ID: &str = "DDS:Crypto:AES_GCM_GMAC";
const CRYPTO_TOKEN_KEYMAT_NAME: &str = "dds.cryp.keymat";

/// DDS:Crypto:AES-GCM-GMAC CryptoToken type from section 9.5.2.1 of the
/// Security specification (v. 1.1)
pub struct BuiltinCryptoToken {
  pub key_material: KeyMaterial_AES_GCM_GMAC,
}
impl TryFrom<CryptoToken> for BuiltinCryptoToken {
  type Error = SecurityError;
  fn try_from(value: CryptoToken) -> Result<Self, Self::Error> {
    let dh = value.data_holder;
    match (dh.class_id.as_str(), dh.properties.as_slice(), dh.binary_properties.as_slice() ) {

      (CRYPTO_TOKEN_CLASS_ID, [], [bp0]) => 
        Ok(Self { key_material: KeyMaterial_AES_GCM_GMAC::try_from(bp0.value.clone())? }),

      (CRYPTO_TOKEN_CLASS_ID, [], bps) =>
        Err(Self::Error 
          { msg: format!("CryptoToken has wrong binary_properties. Expected exactly 1 binary property with name {}.", 
              CRYPTO_TOKEN_KEYMAT_NAME), 
          }),
      (CRYPTO_TOKEN_CLASS_ID, ps, _) =>
        Err(Self::Error 
        { msg: String::from("CryptoToken has wrong properties. Expected properties to be empty."),
        }),

      (cid,_,_)  => 
        Err(Self::Error 
          { msg: format!("CryptoToken has wrong class_id. Expected {}",
                CRYPTO_TOKEN_CLASS_ID ) 
          } ),

    }

  //   if value.data_holder.class_id.ne(CRYPTO_TOKEN_CLASS_ID) {
  //     return Err(Self::Error {
  //       msg: format!(
  //         "CryptoToken has wrong class_id. Expected {}",
  //         CRYPTO_TOKEN_CLASS_ID
  //       ),
  //     });
  //   }
  //   if !value.data_holder.properties.is_empty() {
  //     return Err(Self::Error {
  //       msg: String::from("CryptoToken has wrong properties. Expected properties to be empty."),
  //     });
  //   }
  //   let binary_properties = value.data_holder.binary_properties;
  //   if binary_properties.len() != 1 // There should be exactly one binary property and it should contain the key material
  //     || binary_properties[0]
  //       .name
  //       .ne(CRYPTO_TOKEN_KEYMAT_NAME)
  //   {
  //     return Err(Self::Error {
  //           msg: format!("CryptoToken has wrong binary_properties. Expected exactly 1 binary property with name {}.",CRYPTO_TOKEN_KEYMAT_NAME),
  //         });
  //   }
  //   Ok(Self {
  //     key_material: KeyMaterial_AES_GCM_GMAC::try_from(binary_properties[0].value.clone())?,
  //   })
  }
}

impl TryFrom<BuiltinCryptoToken> for CryptoToken {
  type Error = SecurityError;
  fn try_from(value: BuiltinCryptoToken) -> Result<Self, Self::Error> {
    Ok(CryptoToken {
      data_holder: DataHolder {
        class_id: String::from(CRYPTO_TOKEN_CLASS_ID),
        properties: Vec::new(),
        binary_properties: Vec::from([BinaryProperty {
          name: String::from(CRYPTO_TOKEN_KEYMAT_NAME),
          value: value.key_material.try_into()?,
          propagate: true,
        }]),
      },
    })
  }
}

/// KeyMaterial_AES_GCM_GMAC type from section 9.5.2.1.1 of the Security
/// specification (v. 1.1)
#[allow(non_camel_case_types)] // We use the name from the spec
pub struct KeyMaterial_AES_GCM_GMAC {
  pub transformation_kind: BuiltinCryptoTransformationKind,
  pub master_salt: Vec<u8>,
  pub sender_key_id: CryptoTransformKeyId,
  pub master_sender_key: Vec<u8>,
  pub receiver_specific_key_id: CryptoTransformKeyId,
  pub master_receiver_specific_key: Vec<u8>,
}
impl TryFrom<Bytes> for KeyMaterial_AES_GCM_GMAC {
  type Error = SecurityError;
  fn try_from(value: Bytes) -> Result<Self, Self::Error> {
    Serialized_KeyMaterial_AES_GCM_GMAC::deserialize(&mut CdrDeserializer::<
      BigEndian, /* What's the point of this constructor if we need to specify the byte order
                  * anyway */
    >::new_big_endian(value.as_ref()))
    .map_err(
      // Map deserialization error to SecurityError
      |e| Self::Error {
        msg: format!("Error deserializing KeyMaterial_AES_GCM_GMAC: {}", e),
      },
    )
    .and_then(
      //map transformation_kind to builtin
      |Serialized_KeyMaterial_AES_GCM_GMAC {
         transformation_kind,
         master_salt,
         sender_key_id,
         master_sender_key,
         receiver_specific_key_id,
         master_receiver_specific_key,
       }| {
        BuiltinCryptoTransformationKind::try_from(transformation_kind).map(|transformation_kind| {
          Self {
            transformation_kind,
            master_salt,
            sender_key_id,
            master_sender_key,
            receiver_specific_key_id,
            master_receiver_specific_key,
          }
        })
      },
    )
  }
}
impl TryFrom<KeyMaterial_AES_GCM_GMAC> for Bytes {
  type Error = SecurityError;
  fn try_from(
    KeyMaterial_AES_GCM_GMAC {
      transformation_kind,
      master_salt,
      sender_key_id,
      master_sender_key,
      receiver_specific_key_id,
      master_receiver_specific_key,
    }: KeyMaterial_AES_GCM_GMAC,
  ) -> Result<Self, Self::Error> {
    let transformation_kind = transformation_kind.into();
    let keymat = Serialized_KeyMaterial_AES_GCM_GMAC {
      transformation_kind,
      master_salt,
      sender_key_id,
      master_sender_key,
      receiver_specific_key_id,
      master_receiver_specific_key,
    };
    to_bytes::<Serialized_KeyMaterial_AES_GCM_GMAC, BigEndian>(&keymat)
      .map(Bytes::from)
      .map_err(|e| Self::Error {
        msg: format!("Error serializing KeyMaterial_AES_GCM_GMAC: {}", e),
      })
  }
}

//For (de)serialization
#[allow(non_camel_case_types)] // We use the name from the spec
#[derive(Deserialize, Serialize, PartialEq)]
struct Serialized_KeyMaterial_AES_GCM_GMAC {
  pub transformation_kind: CryptoTransformKind,
  pub master_salt: Vec<u8>,
  pub sender_key_id: CryptoTransformKeyId,
  pub master_sender_key: Vec<u8>,
  pub receiver_specific_key_id: CryptoTransformKeyId,
  pub master_receiver_specific_key: Vec<u8>,
}

/// Valid values for CryptoTransformKind from section 9.5.2.1.1 of the Security
/// specification (v. 1.1)
#[allow(non_camel_case_types)] // We use the names from the spec
pub enum BuiltinCryptoTransformationKind {
  CRYPTO_TRANSFORMATION_KIND_NONE,
  CRYPTO_TRANSFORMATION_KIND_AES128_GMAC,
  CRYPTO_TRANSFORMATION_KIND_AES128_GCM,
  CRYPTO_TRANSFORMATION_KIND_AES256_GMAC,
  CRYPTO_TRANSFORMATION_KIND_AES256_GCM,
}
impl TryFrom<CryptoTransformKind> for BuiltinCryptoTransformationKind {
  type Error = SecurityError;
  fn try_from(value: CryptoTransformKind) -> Result<Self, Self::Error> {
    match value {
      [0, 0, 0, 0] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_NONE),
      [0, 0, 0, 1] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_AES128_GMAC),
      [0, 0, 0, 2] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_AES128_GCM),
      [0, 0, 0, 3] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_AES256_GMAC),
      [0, 0, 0, 4] => Ok(Self::CRYPTO_TRANSFORMATION_KIND_AES256_GCM),
      _ => Err(Self::Error {
        msg: String::from("Invalid CryptoTransformKind"),
      }),
    }
  }
}
impl From<BuiltinCryptoTransformationKind> for CryptoTransformKind {
  fn from(builtin: BuiltinCryptoTransformationKind) -> CryptoTransformKind {
    match builtin {
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_NONE => [0, 0, 0, 0],
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_AES128_GMAC => [0, 0, 0, 1],
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_AES128_GCM => [0, 0, 0, 2],
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_AES256_GMAC => [0, 0, 0, 3],
      BuiltinCryptoTransformationKind::CRYPTO_TRANSFORMATION_KIND_AES256_GCM => [0, 0, 0, 4],
    }
  }
}

/// CryptoTransformIdentifier type from section 9.5.2.2 of the Security
/// specification (v. 1.1)
pub struct BuiltinCryptoTransformIdentifier {
  pub transformation_kind: BuiltinCryptoTransformationKind,
  pub transformation_key_id: CryptoTransformKeyId,
}
impl TryFrom<CryptoTransformIdentifier> for BuiltinCryptoTransformIdentifier {
  type Error = SecurityError;
  fn try_from(value: CryptoTransformIdentifier) -> Result<Self, Self::Error> {
    match BuiltinCryptoTransformationKind::try_from(value.transformation_kind) {
      Err(e) => Err(e),
      Ok(transformation_kind) => Ok(Self {
        transformation_kind,
        transformation_key_id: value.transformation_key_id,
      }),
    }
  }
}

/// CryptoHeader type from section 9.5.2.3 of the Security specification (v.
/// 1.1)
pub struct BuiltinCryptoHeader {
  pub transform_identifier: BuiltinCryptoTransformIdentifier,
  pub session_id: [u8; 4],
  pub initialization_vector_suffix: [u8; 8],
}
impl TryFrom<CryptoHeader> for BuiltinCryptoHeader {
  type Error = SecurityError;
  fn try_from(value: CryptoHeader) -> Result<Self, Self::Error> {
    let crypto_header_extra = value.plugin_crypto_header_extra.data;
    //Try to cast [CryptoTransformIdentifier] to [BuiltinCryptoTransformIdentifier]
    // and read 'session_id' and 'initialization_vector_suffix' from
    // 'crypto_header_extra'
    match (
      BuiltinCryptoTransformIdentifier::try_from(value.transformation_id),
      <[u8; 4]>::try_from(&crypto_header_extra[..4]),
      <[u8; 8]>::try_from(&crypto_header_extra[4..]),
    ) {
      (Err(e), _, _) => Err(e),
      (Ok(transform_identifier), Ok(session_id), Ok(initialization_vector_suffix)) => Ok(Self {
        transform_identifier,
        session_id,
        initialization_vector_suffix,
      }),
      _ => Err(Self::Error {
        msg: format!(
          "plugin_crypto_header_extra was of length {}. Expected 12.",
          crypto_header_extra.len()
        ),
      }),
    }
  }
}

/// CryptoContent type from section 9.5.2.4 of the Security specification (v.
/// 1.1)
pub struct BuiltinCryptoContent {
  pub crypto_content: Vec<u8>,
}
impl TryFrom<CryptoContent> for BuiltinCryptoContent {
  type Error = SecurityError;
  fn try_from(value: CryptoContent) -> Result<Self, Self::Error> {
    todo!();
  }
}

/// CryptoFooter type from section 9.5.2.5 of the Security specification (v.
/// 1.1)
pub struct BuiltinCryptoFooter {
  pub common_mac: [u8; 16],
  pub receiver_specific_macs: Vec<ReceiverSpecificMAC>,
}
impl TryFrom<CryptoFooter> for BuiltinCryptoFooter {
  type Error = SecurityError;
  fn try_from(value: CryptoFooter) -> Result<Self, Self::Error> {
    todo!();
  }
}

/// ReceiverSpecificMAC type from section 9.5.2.5 of the Security specification
/// (v. 1.1)
pub struct ReceiverSpecificMAC {
  pub receiver_mac_key_id: CryptoTransformKeyId,
  pub receiver_mac: [u8; 16],
}
