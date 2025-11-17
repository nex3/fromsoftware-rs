use std::sync::LazyLock;

use pelite::{pattern, pattern::Atom, pe64::Pe};
use shared::{
    util::IncompleteArrayField, FromStatic, InstanceError, InstanceResult, OwnedPtr, Program,
    RecurringTask, SharedTaskImp,
};

use super::PlayerGameData;
use crate::rva;

static GAME_DATA_MAN_PTR_VA: LazyLock<Option<u64>> = LazyLock::new(|| {
    Program::current()
        .rva_to_va(rva::get().game_data_man_ptr)
        .ok()
});

#[repr(C)]
/// Source of name: RTTI
pub struct GameDataMan {
    _vftable: usize,
    _trophy_equip_data: usize,
    pub main_player_game_data: OwnedPtr<PlayerGameData>,
    pub network_players: OwnedPtr<[PlayerGameData; 5]>,
    _unk20: [u8; 0x38],
    _game_settings: usize,
    _menu_system_save_load: usize,
    _profile_summary: usize,
    _pc_option_data: usize,
    _unk78: [u8; 0xB8],
}

impl FromStatic for GameDataMan {
    /// Returns the singleton instance of `GameDataMan`.
    unsafe fn instance() -> InstanceResult<&'static mut Self> {
        let Some(va) = *GAME_DATA_MAN_PTR_VA else {
            return Err(InstanceError::NotFound);
        };
        let pointer = *(va as *const *mut Self);
        unsafe { pointer.as_mut() }.ok_or(InstanceError::Null)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn proper_sizes() {
        assert_eq!(0x130, size_of::<GameDataMan>());
    }
}
