use crate::{
    eip4844::{
        Blob, BlobAndProofV2, BlobTransactionSidecar, Bytes48, BYTES_PER_BLOB,
        BYTES_PER_COMMITMENT, BYTES_PER_PROOF,
    },
    eip7594::{CELLS_PER_EXT_BLOB, EIP_7594_WRAPPER_VERSION},
};
use alloc::{boxed::Box, vec::Vec};
use alloy_primitives::B256;
use alloy_rlp::{BufMut, Decodable, Encodable, Header};

use super::{Decodable7594, Encodable7594};
#[cfg(feature = "kzg")]
use crate::eip4844::BlobTransactionValidationError;
use crate::eip4844::VersionedHashIter;

/// This represents a set of blobs, and its corresponding commitments and proofs.
/// Proof type depends on the sidecar variant.
///
/// This type encodes and decodes the fields without an rlp header.
#[derive(Clone, PartialEq, Eq, Hash, Debug, derive_more::From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(any(test, feature = "arbitrary"), derive(arbitrary::Arbitrary))]
pub enum BlobTransactionSidecarVariant {
    /// EIP-4844 style blob transaction sidecar.
    Eip4844(BlobTransactionSidecar),
    /// EIP-7594 style blob transaction sidecar with cell proofs.
    Eip7594(BlobTransactionSidecarEip7594),
}

impl BlobTransactionSidecarVariant {
    /// Returns true if this is a [`BlobTransactionSidecarVariant::Eip4844`].
    pub const fn is_eip4844(&self) -> bool {
        matches!(self, Self::Eip4844(_))
    }

    /// Returns true if this is a [`BlobTransactionSidecarVariant::Eip7594`].
    pub const fn is_eip7594(&self) -> bool {
        matches!(self, Self::Eip7594(_))
    }

    /// Returns the EIP-4844 sidecar if it is [`Self::Eip4844`].
    pub const fn as_eip4844(&self) -> Option<&BlobTransactionSidecar> {
        match self {
            Self::Eip4844(sidecar) => Some(sidecar),
            _ => None,
        }
    }

    /// Returns the EIP-7594 sidecar if it is [`Self::Eip7594`].
    pub const fn as_eip7594(&self) -> Option<&BlobTransactionSidecarEip7594> {
        match self {
            Self::Eip7594(sidecar) => Some(sidecar),
            _ => None,
        }
    }

    /// Converts into EIP-4844 sidecar if it is [`Self::Eip4844`].
    pub fn into_eip4844(self) -> Option<BlobTransactionSidecar> {
        match self {
            Self::Eip4844(sidecar) => Some(sidecar),
            _ => None,
        }
    }

    /// Converts the EIP-7594 sidecar if it is [`Self::Eip7594`].
    pub fn into_eip7594(self) -> Option<BlobTransactionSidecarEip7594> {
        match self {
            Self::Eip7594(sidecar) => Some(sidecar),
            _ => None,
        }
    }

    /// Calculates a size heuristic for the in-memory size of the [BlobTransactionSidecarVariant].
    #[inline]
    pub fn size(&self) -> usize {
        match self {
            Self::Eip4844(sidecar) => sidecar.size(),
            Self::Eip7594(sidecar) => sidecar.size(),
        }
    }

    /// Verifies that the sidecar is valid. See relevant methods for each variant for more info.
    #[cfg(feature = "kzg")]
    pub fn validate(
        &self,
        blob_versioned_hashes: &[B256],
        proof_settings: &c_kzg::KzgSettings,
    ) -> Result<(), BlobTransactionValidationError> {
        match self {
            Self::Eip4844(sidecar) => sidecar.validate(blob_versioned_hashes, proof_settings),
            Self::Eip7594(sidecar) => sidecar.validate(blob_versioned_hashes, proof_settings),
        }
    }

    /// Returns the commitments of the sidecar.
    pub fn commitments(&self) -> &[Bytes48] {
        match self {
            Self::Eip4844(sidecar) => &sidecar.commitments,
            Self::Eip7594(sidecar) => &sidecar.commitments,
        }
    }

    /// Returns an iterator over the versioned hashes of the commitments.
    pub fn versioned_hashes(&self) -> VersionedHashIter<'_> {
        VersionedHashIter::new(self.commitments())
    }

    /// Returns the index of the versioned hash in the commitments vector.
    pub fn versioned_hash_index(&self, hash: &B256) -> Option<usize> {
        match self {
            Self::Eip4844(s) => s.versioned_hash_index(hash),
            Self::Eip7594(s) => s.versioned_hash_index(hash),
        }
    }

    /// Returns the blob corresponding to the versioned hash, if it exists.
    pub fn blob_by_versioned_hash(&self, hash: &B256) -> Option<&Blob> {
        match self {
            Self::Eip4844(s) => s.blob_by_versioned_hash(hash),
            Self::Eip7594(s) => s.blob_by_versioned_hash(hash),
        }
    }

    /// Outputs the RLP length of the [BlobTransactionSidecarVariant] fields, without a RLP header.
    #[doc(hidden)]
    pub fn rlp_encoded_fields_length(&self) -> usize {
        match self {
            Self::Eip4844(sidecar) => sidecar.rlp_encoded_fields_length(),
            Self::Eip7594(sidecar) => sidecar.rlp_encoded_fields_length(),
        }
    }

    /// Encodes the inner [BlobTransactionSidecarVariant] fields as RLP bytes, __without__ a RLP
    /// header.
    #[inline]
    #[doc(hidden)]
    pub fn rlp_encode_fields(&self, out: &mut dyn BufMut) {
        match self {
            Self::Eip4844(sidecar) => sidecar.rlp_encode_fields(out),
            Self::Eip7594(sidecar) => sidecar.rlp_encode_fields(out),
        }
    }

    /// RLP decode the fields of a [BlobTransactionSidecarVariant] based on the wrapper version.
    #[doc(hidden)]
    pub fn rlp_decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Self::decode_7594(buf)
    }
}

impl Encodable for BlobTransactionSidecarVariant {
    /// Encodes the [BlobTransactionSidecar] fields as RLP bytes, without a RLP header.
    fn encode(&self, out: &mut dyn BufMut) {
        match self {
            Self::Eip4844(sidecar) => sidecar.encode(out),
            Self::Eip7594(sidecar) => sidecar.encode(out),
        }
    }

    fn length(&self) -> usize {
        match self {
            Self::Eip4844(sidecar) => sidecar.rlp_encoded_length(),
            Self::Eip7594(sidecar) => sidecar.rlp_encoded_length(),
        }
    }
}

impl Decodable for BlobTransactionSidecarVariant {
    /// Decodes the inner [BlobTransactionSidecar] fields from RLP bytes, without a RLP header.
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::UnexpectedString);
        }
        if buf.len() < header.payload_length {
            return Err(alloy_rlp::Error::InputTooShort);
        }
        let remaining = buf.len();
        let this = Self::rlp_decode_fields(buf)?;
        if buf.len() + header.payload_length != remaining {
            return Err(alloy_rlp::Error::UnexpectedLength);
        }

        Ok(this)
    }
}

impl Encodable7594 for BlobTransactionSidecarVariant {
    fn encode_7594_len(&self) -> usize {
        self.rlp_encoded_fields_length()
    }

    fn encode_7594(&self, out: &mut dyn BufMut) {
        self.rlp_encode_fields(out);
    }
}

impl Decodable7594 for BlobTransactionSidecarVariant {
    fn decode_7594(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        if buf.first() == Some(&EIP_7594_WRAPPER_VERSION) {
            Ok(Self::Eip7594(Decodable7594::decode_7594(buf)?))
        } else {
            Ok(Self::Eip4844(Decodable7594::decode_7594(buf)?))
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for BlobTransactionSidecarVariant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::fmt;

        #[derive(serde::Deserialize, fmt::Debug)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            Blobs,
            Commitments,
            Proofs,
            CellProofs,
        }

        struct VariantVisitor;

        impl<'de> serde::de::Visitor<'de> for VariantVisitor {
            type Value = BlobTransactionSidecarVariant;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .write_str("a valid blob transaction sidecar (EIP-4844 or EIP-7594 variant)")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut blobs = None;
                let mut commitments = None;
                let mut proofs = None;
                let mut cell_proofs = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Blobs => {
                            blobs = Some(crate::eip4844::deserialize_blobs_map(&mut map)?);
                        }
                        Field::Commitments => commitments = Some(map.next_value()?),
                        Field::Proofs => proofs = Some(map.next_value()?),
                        Field::CellProofs => cell_proofs = Some(map.next_value()?),
                    }
                }

                let blobs = blobs.ok_or_else(|| serde::de::Error::missing_field("blobs"))?;
                let commitments =
                    commitments.ok_or_else(|| serde::de::Error::missing_field("commitments"))?;

                match (cell_proofs, proofs) {
                    (Some(cp), None) => {
                        Ok(BlobTransactionSidecarVariant::Eip7594(BlobTransactionSidecarEip7594 {
                            blobs,
                            commitments,
                            cell_proofs: cp,
                        }))
                    }
                    (None, Some(pf)) => {
                        Ok(BlobTransactionSidecarVariant::Eip4844(BlobTransactionSidecar {
                            blobs,
                            commitments,
                            proofs: pf,
                        }))
                    }
                    (None, None) => {
                        Err(serde::de::Error::custom("Missing 'cellProofs' or 'proofs'"))
                    }
                    (Some(_), Some(_)) => Err(serde::de::Error::custom(
                        "Both 'cellProofs' and 'proofs' cannot be present",
                    )),
                }
            }
        }

        const FIELDS: &[&str] = &["blobs", "commitments", "proofs", "cellProofs"];
        deserializer.deserialize_struct("BlobTransactionSidecarVariant", FIELDS, VariantVisitor)
    }
}

/// This represents a set of blobs, and its corresponding commitments and cell proofs.
///
/// This type encodes and decodes the fields without an rlp header.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(test, feature = "arbitrary"), derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BlobTransactionSidecarEip7594 {
    /// The blob data.
    #[cfg_attr(feature = "serde", serde(deserialize_with = "crate::eip4844::deserialize_blobs"))]
    pub blobs: Vec<Blob>,
    /// The blob commitments.
    pub commitments: Vec<Bytes48>,
    /// List of cell proofs for all blobs in the sidecar, including the proofs for the extension
    /// indices, for a total of `CELLS_PER_EXT_BLOB` proofs per blob (`CELLS_PER_EXT_BLOB` is the
    /// number of cells for an extended blob, defined in
    /// [the consensus specs](https://github.com/ethereum/consensus-specs/tree/9d377fd53d029536e57cfda1a4d2c700c59f86bf/specs/fulu/polynomial-commitments-sampling.md#cells))
    pub cell_proofs: Vec<Bytes48>,
}

impl core::fmt::Debug for BlobTransactionSidecarEip7594 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BlobTransactionSidecarEip7594")
            .field("blobs", &self.blobs.len())
            .field("commitments", &self.commitments)
            .field("cell_proofs", &self.cell_proofs)
            .finish()
    }
}

impl BlobTransactionSidecarEip7594 {
    /// Constructs a new [BlobTransactionSidecarEip7594] from a set of blobs, commitments, and
    /// cell proofs.
    pub const fn new(
        blobs: Vec<Blob>,
        commitments: Vec<Bytes48>,
        cell_proofs: Vec<Bytes48>,
    ) -> Self {
        Self { blobs, commitments, cell_proofs }
    }

    /// Calculates a size heuristic for the in-memory size of the [BlobTransactionSidecarEip7594].
    #[inline]
    pub fn size(&self) -> usize {
        self.blobs.len() * BYTES_PER_BLOB + // blobs
               self.commitments.len() * BYTES_PER_COMMITMENT + // commitments
               self.cell_proofs.len() * BYTES_PER_PROOF // proofs
    }

    /// Verifies that the versioned hashes are valid for this sidecar's blob data, commitments, and
    /// proofs.
    ///
    /// Takes as input the [KzgSettings](c_kzg::KzgSettings), which should contain the parameters
    /// derived from the KZG trusted setup.
    ///
    /// This ensures that the blob transaction payload has the expected number of blob data
    /// elements, commitments, and proofs. The cells are constructed from each blob and verified
    /// against the commitments and proofs.
    ///
    /// Returns [BlobTransactionValidationError::InvalidProof] if any blob KZG proof in the response
    /// fails to verify, or if the versioned hashes in the transaction do not match the actual
    /// commitment versioned hashes.
    #[cfg(feature = "kzg")]
    pub fn validate(
        &self,
        blob_versioned_hashes: &[B256],
        proof_settings: &c_kzg::KzgSettings,
    ) -> Result<(), BlobTransactionValidationError> {
        // Ensure the versioned hashes and commitments have the same length.
        if blob_versioned_hashes.len() != self.commitments.len() {
            return Err(c_kzg::Error::MismatchLength(format!(
                "There are {} versioned commitment hashes and {} commitments",
                blob_versioned_hashes.len(),
                self.commitments.len()
            ))
            .into());
        }

        let blobs_len = self.blobs.len();
        let expected_cell_proofs_len = blobs_len * CELLS_PER_EXT_BLOB;
        if self.cell_proofs.len() != expected_cell_proofs_len {
            return Err(c_kzg::Error::MismatchLength(format!(
                "There are {} cell proofs and {} blobs. Expected {} cell proofs.",
                self.cell_proofs.len(),
                blobs_len,
                expected_cell_proofs_len
            ))
            .into());
        }

        // calculate versioned hashes by zipping & iterating
        for (versioned_hash, commitment) in
            blob_versioned_hashes.iter().zip(self.commitments.iter())
        {
            // calculate & verify versioned hash
            let calculated_versioned_hash =
                crate::eip4844::kzg_to_versioned_hash(commitment.as_slice());
            if *versioned_hash != calculated_versioned_hash {
                return Err(BlobTransactionValidationError::WrongVersionedHash {
                    have: *versioned_hash,
                    expected: calculated_versioned_hash,
                });
            }
        }

        // Repeat cell ranges for each blob.
        let cell_indices =
            Vec::from_iter((0..blobs_len).flat_map(|_| 0..CELLS_PER_EXT_BLOB as u64));

        // Repeat commitments for each cell.
        let mut commitments = Vec::with_capacity(blobs_len * CELLS_PER_EXT_BLOB);
        for commitment in &self.commitments {
            commitments.extend(core::iter::repeat_n(*commitment, CELLS_PER_EXT_BLOB));
        }

        // SAFETY: ALL types have the same size
        let res = unsafe {
            let mut cells = Vec::with_capacity(blobs_len * CELLS_PER_EXT_BLOB);
            for blob in &self.blobs {
                let blob = core::mem::transmute::<&Blob, &c_kzg::Blob>(blob);
                cells.extend(proof_settings.compute_cells(blob)?.into_iter());
            }

            proof_settings.verify_cell_kzg_proof_batch(
                // commitments
                core::mem::transmute::<&[Bytes48], &[c_kzg::Bytes48]>(&commitments),
                // cell indices
                &cell_indices,
                // cells
                &cells,
                // proofs
                core::mem::transmute::<&[Bytes48], &[c_kzg::Bytes48]>(self.cell_proofs.as_slice()),
            )?
        };

        res.then_some(()).ok_or(BlobTransactionValidationError::InvalidProof)
    }

    /// Returns an iterator over the versioned hashes of the commitments.
    pub fn versioned_hashes(&self) -> VersionedHashIter<'_> {
        VersionedHashIter::new(&self.commitments)
    }

    /// Returns the index of the versioned hash in the commitments vector.
    pub fn versioned_hash_index(&self, hash: &B256) -> Option<usize> {
        self.commitments.iter().position(|commitment| {
            crate::eip4844::kzg_to_versioned_hash(commitment.as_slice()) == *hash
        })
    }

    /// Returns the blob corresponding to the versioned hash, if it exists.
    pub fn blob_by_versioned_hash(&self, hash: &B256) -> Option<&Blob> {
        self.versioned_hash_index(hash).and_then(|index| self.blobs.get(index))
    }

    /// Matches versioned hashes and returns an iterator of (index, [`BlobAndProofV2`]) pairs
    /// where index is the position in `versioned_hashes` that matched the versioned hash in the
    /// sidecar.
    ///
    /// This is used for the `engine_getBlobsV2` RPC endpoint of the engine API
    pub fn match_versioned_hashes<'a>(
        &'a self,
        versioned_hashes: &'a [B256],
    ) -> impl Iterator<Item = (usize, BlobAndProofV2)> + 'a {
        self.versioned_hashes().enumerate().flat_map(move |(i, blob_versioned_hash)| {
            versioned_hashes.iter().enumerate().filter_map(move |(j, target_hash)| {
                if blob_versioned_hash == *target_hash {
                    let maybe_blob = self.blobs.get(i);
                    let proof_range = i * CELLS_PER_EXT_BLOB..(i + 1) * CELLS_PER_EXT_BLOB;
                    let maybe_proofs = Some(&self.cell_proofs[proof_range])
                        .filter(|proofs| proofs.len() == CELLS_PER_EXT_BLOB);
                    if let Some((blob, proofs)) = maybe_blob.copied().zip(maybe_proofs) {
                        return Some((
                            j,
                            BlobAndProofV2 { blob: Box::new(blob), proofs: proofs.to_vec() },
                        ));
                    }
                }
                None
            })
        })
    }

    /// Outputs the RLP length of [BlobTransactionSidecarEip7594] fields without a RLP header.
    #[doc(hidden)]
    pub fn rlp_encoded_fields_length(&self) -> usize {
        // wrapper version + blobs + commitments + cell proofs
        1 + self.blobs.length() + self.commitments.length() + self.cell_proofs.length()
    }

    /// Encodes the inner [BlobTransactionSidecarEip7594] fields as RLP bytes, __without__ a
    /// RLP header.
    ///
    /// This encodes the fields in the following order:
    /// - `wrapper_version`
    /// - `blobs`
    /// - `commitments`
    /// - `cell_proofs`
    #[inline]
    #[doc(hidden)]
    pub fn rlp_encode_fields(&self, out: &mut dyn BufMut) {
        // Put version byte.
        out.put_u8(EIP_7594_WRAPPER_VERSION);
        // Encode the blobs, commitments, and cell proofs
        self.blobs.encode(out);
        self.commitments.encode(out);
        self.cell_proofs.encode(out);
    }

    /// Creates an RLP header for the [BlobTransactionSidecarEip7594].
    fn rlp_header(&self) -> Header {
        Header { list: true, payload_length: self.rlp_encoded_fields_length() }
    }

    /// Calculates the length of the [BlobTransactionSidecarEip7594] when encoded as
    /// RLP.
    pub fn rlp_encoded_length(&self) -> usize {
        self.rlp_header().length() + self.rlp_encoded_fields_length()
    }

    /// Encodes the [BlobTransactionSidecarEip7594] as RLP bytes.
    pub fn rlp_encode(&self, out: &mut dyn BufMut) {
        self.rlp_header().encode(out);
        self.rlp_encode_fields(out);
    }

    /// RLP decode the fields of a [BlobTransactionSidecarEip7594].
    #[doc(hidden)]
    pub fn rlp_decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            blobs: Decodable::decode(buf)?,
            commitments: Decodable::decode(buf)?,
            cell_proofs: Decodable::decode(buf)?,
        })
    }

    /// Decodes the [BlobTransactionSidecarEip7594] from RLP bytes.
    pub fn rlp_decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::UnexpectedString);
        }
        if buf.len() < header.payload_length {
            return Err(alloy_rlp::Error::InputTooShort);
        }
        let remaining = buf.len();

        let this = Self::decode_7594(buf)?;
        if buf.len() + header.payload_length != remaining {
            return Err(alloy_rlp::Error::UnexpectedLength);
        }

        Ok(this)
    }
}

impl Encodable for BlobTransactionSidecarEip7594 {
    /// Encodes the inner [BlobTransactionSidecarEip7594] fields as RLP bytes, without a RLP header.
    fn encode(&self, out: &mut dyn BufMut) {
        self.rlp_encode(out);
    }

    fn length(&self) -> usize {
        self.rlp_encoded_length()
    }
}

impl Decodable for BlobTransactionSidecarEip7594 {
    /// Decodes the inner [BlobTransactionSidecarEip7594] fields from RLP bytes, without a RLP
    /// header.
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Self::rlp_decode(buf)
    }
}

impl Encodable7594 for BlobTransactionSidecarEip7594 {
    fn encode_7594_len(&self) -> usize {
        self.rlp_encoded_fields_length()
    }

    fn encode_7594(&self, out: &mut dyn BufMut) {
        self.rlp_encode_fields(out);
    }
}

impl Decodable7594 for BlobTransactionSidecarEip7594 {
    fn decode_7594(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let wrapper_version: u8 = Decodable::decode(buf)?;
        if wrapper_version != EIP_7594_WRAPPER_VERSION {
            return Err(alloy_rlp::Error::Custom("invalid wrapper version"));
        }
        Self::rlp_decode_fields(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_variant_rlp_roundtrip() {
        let mut encoded = Vec::new();

        // 4844
        let empty_sidecar_4844 =
            BlobTransactionSidecarVariant::Eip4844(BlobTransactionSidecar::default());
        empty_sidecar_4844.encode(&mut encoded);
        assert_eq!(
            empty_sidecar_4844,
            BlobTransactionSidecarVariant::decode(&mut &encoded[..]).unwrap()
        );

        let sidecar_4844 = BlobTransactionSidecarVariant::Eip4844(BlobTransactionSidecar::new(
            vec![Blob::default()],
            vec![Bytes48::ZERO],
            vec![Bytes48::ZERO],
        ));
        encoded.clear();
        sidecar_4844.encode(&mut encoded);
        assert_eq!(sidecar_4844, BlobTransactionSidecarVariant::decode(&mut &encoded[..]).unwrap());

        // 7594
        let empty_sidecar_7594 =
            BlobTransactionSidecarVariant::Eip7594(BlobTransactionSidecarEip7594::default());
        encoded.clear();
        empty_sidecar_7594.encode(&mut encoded);
        assert_eq!(
            empty_sidecar_7594,
            BlobTransactionSidecarVariant::decode(&mut &encoded[..]).unwrap()
        );

        let sidecar_7594 =
            BlobTransactionSidecarVariant::Eip7594(BlobTransactionSidecarEip7594::new(
                vec![Blob::default()],
                vec![Bytes48::ZERO],
                core::iter::repeat_n(Bytes48::ZERO, CELLS_PER_EXT_BLOB).collect(),
            ));
        encoded.clear();
        sidecar_7594.encode(&mut encoded);
        assert_eq!(sidecar_7594, BlobTransactionSidecarVariant::decode(&mut &encoded[..]).unwrap());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn sidecar_variant_json_deserialize_sanity() {
        let mut eip4844 = BlobTransactionSidecar::default();
        eip4844.blobs.push(Blob::repeat_byte(0x2));

        let json = serde_json::to_string(&eip4844).unwrap();
        let variant: BlobTransactionSidecarVariant = serde_json::from_str(&json).unwrap();
        assert!(variant.is_eip4844());
        let jsonvariant = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, jsonvariant);

        let mut eip7594 = BlobTransactionSidecarEip7594::default();
        eip7594.blobs.push(Blob::repeat_byte(0x4));
        let json = serde_json::to_string(&eip7594).unwrap();
        let variant: BlobTransactionSidecarVariant = serde_json::from_str(&json).unwrap();
        assert!(variant.is_eip7594());
        let jsonvariant = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, jsonvariant);
    }

    #[test]
    fn rlp_7594_roundtrip() {
        let mut encoded = Vec::new();

        let sidecar_4844 = BlobTransactionSidecar::default();
        sidecar_4844.encode_7594(&mut encoded);
        assert_eq!(sidecar_4844, Decodable7594::decode_7594(&mut &encoded[..]).unwrap());

        let sidecar_variant_4844 = BlobTransactionSidecarVariant::Eip4844(sidecar_4844);
        assert_eq!(sidecar_variant_4844, Decodable7594::decode_7594(&mut &encoded[..]).unwrap());
        encoded.clear();
        sidecar_variant_4844.encode_7594(&mut encoded);
        assert_eq!(sidecar_variant_4844, Decodable7594::decode_7594(&mut &encoded[..]).unwrap());

        let sidecar_7594 = BlobTransactionSidecarEip7594::default();
        encoded.clear();
        sidecar_7594.encode_7594(&mut encoded);
        assert_eq!(sidecar_7594, Decodable7594::decode_7594(&mut &encoded[..]).unwrap());

        let sidecar_variant_7594 = BlobTransactionSidecarVariant::Eip7594(sidecar_7594);
        assert_eq!(sidecar_variant_7594, Decodable7594::decode_7594(&mut &encoded[..]).unwrap());
        encoded.clear();
        sidecar_variant_7594.encode_7594(&mut encoded);
        assert_eq!(sidecar_variant_7594, Decodable7594::decode_7594(&mut &encoded[..]).unwrap());
    }
}
