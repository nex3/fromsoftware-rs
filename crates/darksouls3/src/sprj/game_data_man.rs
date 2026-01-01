use std::{borrow::Cow, sync::LazyLock};

use pelite::pe64::Pe;
use shared::{FromStatic, InstanceError, InstanceResult, OwnedPtr, Program};

use super::{ItemId, PlayerGameData};
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

static LUA_EVENT_MAN_GIVE_ITEM_DIRECTLY_VA: LazyLock<u64> = LazyLock::new(|| {
    Program::current()
        .rva_to_va(rva::get().lua_event_man_give_item_directly)
        .expect("Call target for LUA_EVENT_MAN_GIVE_ITEM_DIRECTLY_VA was not in exe")
});

impl GameDataMan {
    /// Gives the player [quantity] instances of [item].
    ///
    /// Note that this won't give more than one copy of certain key items.
    pub fn give_item_directly(&mut self, item: ItemId, quantity: u32) {
        // Because this function comes from the event manager, it takes the
        // LuaEventMan as its first argument rather than GameDataMan. It instead
        // accesses GameDataMan through the global variable. To avoid needing to
        // mark this function unsafe, though, we make it a method on
        // `GameDataMan` anyway. Since there's only one instance of this
        // globally, if we have a mutable reference to it we know it's safe to
        // run code that modifies it through the global variable.
        let give_item_directly: extern "C" fn(usize, u32, u32, u32) =
            unsafe { std::mem::transmute(*LUA_EVENT_MAN_GIVE_ITEM_DIRECTLY_VA) };

        // The LuaEventMan isn't actually used.
        give_item_directly(0, (item.category() as u32) << 28, item.param_id(), quantity);
    }

    /// Removes [quantity] instances of [item] from the player's inventory.
    pub fn remove_item(&mut self, item: ItemId, quantity: u32) {
        // As above, this takes LuaEventMan but doesn't use it.
        let rva = Program::current()
            .rva_to_va(rva::get().lua_event_man_remove_item)
            .unwrap();
        let remove_item: extern "C" fn(usize, u32, u32, u32) = unsafe { std::mem::transmute(rva) };

        remove_item(0, (item.category() as u32) << 28, item.param_id(), quantity);
    }
}

impl FromStatic for GameDataMan {
    fn name() -> Cow<'static, str> {
        "GameDataMan".into()
    }

    /// Returns the singleton instance of `GameDataMan`.
    unsafe fn instance() -> InstanceResult<&'static mut Self> {
        let Some(va) = *GAME_DATA_MAN_PTR_VA else {
            return Err(InstanceError::NotFound);
        };
        let pointer = unsafe { *(va as *const *mut Self) };
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
