use crate::db::errors::DataError;
use byteorder::{BigEndian, LittleEndian};
use zerocopy::byteorder::{I32, U32, U64};
use zerocopy::LayoutVerified;

/// User variables are stored as little-endian 32-bit integers in the
/// database. This type alias makes the database code more pleasant to
/// read.
type LittleEndianI32Layout<'a> = LayoutVerified<&'a [u8], I32<LittleEndian>>;

type LittleEndianU32Layout<'a> = LayoutVerified<&'a [u8], U32<LittleEndian>>;

#[allow(dead_code)]
type LittleEndianU64Layout<'a> = LayoutVerified<&'a [u8], U64<LittleEndian>>;

type BigEndianU64Layout<'a> = LayoutVerified<&'a [u8], U64<BigEndian>>;

/// Convert bytes to an i32 with zero-copy deserialization. An error
/// is returned if the bytes do not represent an i32.
pub(super) fn convert_i32(raw_value: &[u8]) -> Result<i32, DataError> {
    let layout = LittleEndianI32Layout::new_unaligned(raw_value.as_ref());

    if let Some(layout) = layout {
        let value: I32<LittleEndian> = *layout;
        Ok(value.get())
    } else {
        Err(DataError::I32SchemaViolation)
    }
}

pub(super) fn convert_u32(raw_value: &[u8]) -> Result<u32, DataError> {
    let layout = LittleEndianU32Layout::new_unaligned(raw_value.as_ref());

    if let Some(layout) = layout {
        let value: U32<LittleEndian> = *layout;
        Ok(value.get())
    } else {
        Err(DataError::I32SchemaViolation)
    }
}

#[allow(dead_code)]
pub(super) fn convert_u64(raw_value: &[u8]) -> Result<u64, DataError> {
    let layout = BigEndianU64Layout::new_unaligned(raw_value.as_ref());

    if let Some(layout) = layout {
        let value: U64<BigEndian> = *layout;
        Ok(value.get())
    } else {
        Err(DataError::I32SchemaViolation)
    }
}
