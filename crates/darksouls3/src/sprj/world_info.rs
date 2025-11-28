use std::mem::MaybeUninit;
use std::ptr::NonNull;

use shared::UnknownStruct;

#[repr(C)]
/// Source of name: RTTI
pub struct WorldInfo {
    _vftable: usize,

    /// The number of defined entries in [world_area_info].
    pub world_area_info_count: u32,

    /// A pointer to the beginning of [world_area_info].
    pub world_area_info_list_ptr: NonNull<WorldAreaInfo>,

    /// The number of defined entries in [world_block_info].
    pub world_block_info_count: u32,

    /// A pointer to the beginning of [world_block_info].
    pub world_block_info_list_ptr: NonNull<WorldBlockInfo>,

    _unk28: u8,

    /// The pool of [WorldAreaInfo]s. Only the first [world_area_info_count]
    /// are initialized.
    pub world_area_info: [MaybeUninit<WorldAreaInfo>; 0x14],

    /// The pool of [WorldBlockInfo]s. Only the first [world_block_info_count]
    /// are initialized.
    pub world_block_info: [MaybeUninit<WorldBlockInfo>; 0x20],

    _unk1290: u64,
}

#[repr(C)]
/// Source of name: RTTI
pub struct WorldAreaInfo {
    _vftable: usize,
    _pad08: [u8; 3],
    pub area_number: u8,

    /// The [WorldInfo] instance that owns this area.
    pub owner: NonNull<WorldInfo>,

    _unk18: u32,
    _unk1c: u32,
    _unk20: u32,
    _unk28: usize,
    _unk30: u8,
}

#[repr(C)]
/// Source of name: RTTI
pub struct WorldBlockInfo {
    _vftable: usize,
    _unk08: u32,

    /// The [WorldInfo] instance that owns this area.
    pub owner: NonNull<WorldInfo>,

    /// The [WorldAreaInfo] that corresponds to this block.
    pub world_area_info: Option<NonNull<WorldAreaInfo>>,

    /// The index of this in [WorldInfo.world_block_info].
    pub world_block_index: u32,

    _unk24: u32,
    _msb_res_cap: usize,
    _btab_file_cap: usize,
    _btl_file_cap: usize,
    _btpb_file_cap: usize,
    _breakobj_file_cap: usize,
    _pre_map_decal_file_cap: usize,
    _unk58: usize,
    _unk60: u8,
    _pad61: [u8; 3],
    _unk64: u8,
    _unk68: u32,
}

#[repr(C)]
/// Source of name: RTTI
pub struct WorldInfoOwner {
    pub super_world_info: WorldInfo,
    _unk8: u64,

    /// The number of defined entries in [world_area_res].
    pub world_area_res_count: u32,

    /// A pointer to the beginning of [world_area_res].
    pub world_area_res_list_ptr: NonNull<WorldAreaRes>,

    _unk12b0: u32,
    _unk12b4: u32,

    /// The number of defined entries in [world_block_res].
    pub world_block_res_count: u32,

    /// A pointer to the beginning of [world_block_res].
    pub world_block_res_list_ptr: NonNull<WorldBlockRes>,

    _unk12c8: u64,
    _unk12d0: u64,
    _unk12d8: u64,

    /// The pool of [WorldAreaRes]es. Only the first [world_area_res_count] are
    /// initialized.
    pub world_area_res: [MaybeUninit<WorldAreaRes>; 0x14],

    /// The pool of [WorldBlockRes]es. Only the first [world_block_res_count]
    /// are initialized.
    pub world_block_res: [MaybeUninit<WorldBlockRes>; 0x20],

    _unkae80: u64,
    _unkae88: u64,
}

/// Source of name: RTTI
type WorldAreaRes = UnknownStruct<0x108>;

/// Source of name: RTTI
type WorldBlockRes = UnknownStruct<0x438>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn proper_sizes() {
        assert_eq!(0x38, size_of::<WorldAreaInfo>());
        assert_eq!(0x70, size_of::<WorldBlockInfo>());
        assert_eq!(0x1298, size_of::<WorldInfo>());
        assert_eq!(0xae90, size_of::<WorldInfoOwner>());
    }
}
