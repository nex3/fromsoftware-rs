use std::mem::MaybeUninit;
use std::ptr::NonNull;

use shared::{OwnedPtr, Subclass, UnknownStruct};

use super::{ChrIns, PlayerIns, ReplayGhostIns, WorldBlockChr, WorldInfoOwner};
use crate::CxxVec;

#[repr(C)]
#[shared::singleton("WorldChrMan")]
/// Source of name: RTTI
pub struct WorldChrMan {
    vtable: usize,
    pub world_info_owner: NonNull<WorldInfoOwner>,

    /// The number of defined entries in [world_area_chr].
    pub world_area_chr_count: u32,

    /// A pointer to the beginning of [world_area_chr].
    pub world_area_chr_ptr: NonNull<WorldAreaChr>,

    /// The number of defined entries in [world_block_chr].
    pub world_block_chr_count: u32,

    /// A pointer to the beginning of [world_block_chr].
    pub world_block_chr_ptr: NonNull<WorldBlockChr>,

    _unk30: u32,

    /// All human players.
    pub player_chr_set: ChrSet<PlayerIns>,

    /// Bloodstain and replay ghosts.
    pub ghost_chr_set: ChrSet<ReplayGhostIns>,

    /// Debug characters. This doesn't seem to be populated in normal gameplay.
    pub debug_chr_set: ChrSet<ChrIns>,

    /// The local player.
    pub main_player: Option<NonNull<PlayerIns>>,

    /// Another player. Maybe the owner of the host world during multiplayer?
    _unk88: Option<NonNull<PlayerIns>>,

    _unk90: u16,
    _unk92: [u8; 0xd],
    _unka0: u64,
    _unka8: u64,
    _unkb0: u64,
    _unkb8: u64,

    /// The length of [loaded_world_block_chr_ptr].
    pub loaded_world_block_chr_count: i32,
    pub loaded_world_block_chr_ptr: [NonNull<WorldBlockChr>; 32],

    _unk1c8: u32,
    _unk1d0: [UnknownStruct<0x18>; 35],
    _unk518: [u8; 0x118],
    _unk630: OwnedPtr<UnknownStruct<0x67c8>>,
    _unk638: OwnedPtr<u8>,
    _unk640: OwnedPtr<u8>,
    _unk648: OwnedPtr<UnknownStruct<0x18>>,
    _chr_thread: usize,
    _unk658: u64,

    /// The pool of [WorldAreaChr]s. Only the first [world_area_chr_count]
    /// are initialized.
    pub world_area_chr: [MaybeUninit<WorldAreaChr>; 20],

    /// The pool of [WorldBlockChr]s. Only the first [world_area_chr_count]
    /// are initialized.
    pub world_block_chr: [MaybeUninit<WorldBlockChr>; 32],

    _unk2fe0: u64,
    _unk2fe8: u64,
    _unk2ff0: i32,
    _unk2ff8: CxxVec<usize>,
    _debug_chr_creator: usize,
    _debug_chr_perf_checker: usize,
    _unk3028: u64,
    _unk3030: u64,
    _allocator: usize,
    _unk3040: u32,
    _unk3048: u32,
    _unk304c: u16,
    _unk3050: CxxVec<usize>,
    _unk3058: u64,
    _unk3060: u64,
    _unk3068: u64,
    _unk3088: [u64; 35],
    _unk31a0: u64,
    _update_tasks: [UnknownStruct<0x30>; 0xc],
    _unk33e8: u32,
    _void_tasks: [UnknownStruct<0x28>; 0xa],
}

#[repr(C)]
/// Source of name: RTTI
pub struct WorldAreaChr {
    _vftable: usize,
    _unk08: u64,
    _unk10: u32,
    _unk18: u64,
}

#[repr(C)]
/// Source of name: Copied from ER RTTI
pub struct ChrSet<T>
where
    T: Subclass<ChrIns>,
{
    /// The size of the set.
    pub length: u32,

    /// The contents of the set.
    pub entries: OwnedPtr<ChrSetEntry<T>>,

    _unk10: u32,
}

#[repr(C)]
/// Source of name: Copied from ER RTTI
pub struct ChrSetEntry<T>
where
    T: Subclass<ChrIns>,
{
    /// The character this entry refers to.
    pub chr: OwnedPtr<T>,

    _unk08: [u8; 0x10],
    _special_effect: usize,
    _unk20: [u8; 8],
    _chr_physics_module: usize,
    _unk30: usize,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn proper_sizes() {
        assert_eq!(0x20, size_of::<WorldAreaChr>());
        assert_eq!(0x38, size_of::<ChrSetEntry<ChrIns>>());
        assert_eq!(0x18, size_of::<ChrSet<ChrIns>>());
        assert_eq!(0x3580, size_of::<WorldChrMan>());
    }
}
