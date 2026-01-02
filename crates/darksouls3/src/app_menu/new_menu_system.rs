use std::{borrow::Cow, ptr::NonNull};

use crate::{CxxVec, dlut::DLFixedVector, rva};
use shared::{FromStatic, UnknownStruct};

#[repr(C)]
pub struct NewMenuSystem {
    _vftable: usize,
    _array_menu_window_job_1: usize,
    _unk10: [u8; 0x30],
    pub windows: DLFixedVector<NonNull<MenuWindow>, 8>,
    _menu_window_job_1: usize,
    _unk98: u64,
    finalize_callback_job: usize,
    _unka8: u64,
    _unkb0: u64,
    _unkb8: u64,
    _unkc0: u64,
    _menu_window_job_2: usize,
    _unkd0: [u8; 0x18],
    _callback: usize,
    _finalize_callback_jobs: DLFixedVector<usize, 8>,
    _unk140: bool,
    _unk144: u32,
    _unk148: u32,
    _unk150: u64,
    _unk158: u16,
    _unk160: UnknownStruct<0x2ec8>,
    _fe_emergency_notice: usize,
    _fe_summon_message: usize,
    _fade_screen: usize,
    _fe_view: usize,
    _unk3048: [u8; 0x28],
    _unk3070: u64,
    _array_menu_window_job_2: usize,
    _unk3080: u8,
    _unk3081: u8,
    _unk3082: u8,
    _unk3084: u16,
    _unk3088: u64,
    _unk3090: u32,
}

impl NewMenuSystem {
    /// Returns whether an in-game menu (including bonfires, shops, and so on)
    /// is currently open. This is always false on the main menu, even if a
    /// settings sub-menu is open.
    pub fn is_menu_open(&self) -> bool {
        // This is a function pointer for a callback used to clean up a menu.
        // It's always set for menus in the main game, but never for other
        // menus.
        self.finalize_callback_job != 0
    }
}

impl FromStatic for NewMenuSystem {
    fn name() -> Cow<'static, str> {
        "NewMenuSystem".into()
    }

    unsafe fn instance() -> fromsoftware_shared::InstanceResult<&'static mut Self> {
        unsafe { shared::load_static_indirect(rva::get().app_menu_new_menu_system_ptr) }
    }
}

#[repr(C)]
pub struct MenuWindow {
    _vftable: usize,
    _unk08: u32,
    _fix_order_job_sequence: usize,
    _unk18: [u8; 0x28],
    _unk40: u64,
    _grid_control: usize,
    _option_setting_top_dialog: usize,
    _unk58: [u8; 0x38],
    pub unk90: u64,
    _unk98: [u8; 0x18],
    _unkb0: u64,
    _unkb8: [u8; 0x18],
    _unkd0: u64,
    _unkd8: SceneObjProxy,
    _unk138: SceneObjProxy,
    _unk198: u64,
    _unk1a0: CxxVec<u64>,
    _unk1c0: CxxVec<u64>,
    _component_holder: usize,
    _unk1e8: [u8; 0x18],
    _unk200: SprjScaleformValue,
    _unk238: SceneObjProxy,
    _unk298: SceneObjProxy,
    _unk2f8: SceneObjProxy,
    _unk358: u8,
    _unk360: SceneObjProxy,
    _unk3c0: [u8; 0x608],
    _unk9c8: u64,
    _unk9d0: u64,
}

type SceneObjProxy = UnknownStruct<0x60>;
type SprjScaleformValue = UnknownStruct<0x38>;

#[cfg(test)]
mod test {
    use crate::app_menu::{MenuWindow, NewMenuSystem};

    #[test]
    fn proper_sizes() {
        assert_eq!(0x9d8, size_of::<MenuWindow>());
        assert_eq!(0x3098, size_of::<NewMenuSystem>());
    }
}
