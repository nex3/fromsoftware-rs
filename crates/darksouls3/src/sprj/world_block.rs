use shared::OwnedPtr;

use crate::sprj::ChrSet;

use super::ChrIns;
use super::FieldInsSelector;

#[repr(C)]
/// Source of name: RTTI
pub struct WorldBlockChr {
    _vftable: usize,
    _unk08: [u64; 0xe],
    _unk78: u32,

    /// The set of character entities associated with this block.
    pub chr_set: ChrSet<ChrIns>,

    _unk98: u32,
    _unka0: u64,

    /// The length of [mappings].
    pub mappings_length: i32,

    /// Mappings from entity IDs to [FileInsSelector]s.
    pub mappings: OwnedPtr<WorldBlockMapping>,

    _unkb8: u32,
    _unkc0: [u64; 5],
    _unke8: [u8; 0x48],
    _unk134: u32,
}

/// A mapping from an entity ID to a [FieldInsSelector].
#[repr(C)]
pub struct WorldBlockMapping {
    /// The entity this mapping refers to.
    pub entity_id: i32,

    /// The selector corresponding to this entity ID.
    pub selector: FieldInsSelector,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn proper_sizes() {
        assert_eq!(0x138, size_of::<WorldBlockChr>());
    }
}
