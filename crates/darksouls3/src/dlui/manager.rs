use std::sync::LazyLock;

use pelite::pe64::Pe;
use shared::{FromStatic, InstanceError, InstanceResult};
use shared::{OwnedPtr, Program};

use crate::dlkr::{DLAllocatorRef, DLPlainLightMutex};
use crate::dlui::DLUserInputDeviceImpl;
use crate::CxxVec;

#[repr(C)]
pub struct DLUserInputManager {
    _vftable: usize,
    pub mutex: DLPlainLightMutex,
    pub allocator: DLAllocatorRef,
    _unk48: u64,
    pub unk50: CxxVec<u64>,
    pub unk70: CxxVec<u64>,
    pub devices: CxxVec<OwnedPtr<DLUserInputDeviceImpl>>,
    _unkb0: u64,
    pub dummy_device: DLUserInputDeviceImpl,
    pub com_initialized: bool,
    _unk249: u32,
    _unk24b: bool,
    _unk24c: bool,
    pub window_active: bool,
    _unk24e: [u8; 0xA],
    _unk258: CxxVec<u64>,
    _unk278: [u8; 0x50],
}

static DL_USER_INPUT_MANAGER_PTR_VA: LazyLock<Option<u64>> =
    LazyLock::new(|| Program::current().rva_to_va(0x49644b8).ok());

impl FromStatic for DLUserInputManager {
    /// Returns the singleton instance of `MapItemMan`.
    unsafe fn instance() -> InstanceResult<&'static mut Self> {
        let Some(va) = *DL_USER_INPUT_MANAGER_PTR_VA else {
            return Err(InstanceError::NotFound);
        };
        let pointer = *(va as *const *mut Self);
        unsafe { pointer.as_mut() }.ok_or(InstanceError::Null)
    }
}
