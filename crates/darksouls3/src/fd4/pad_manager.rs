use shared::OwnedPtr;

use crate::dlui::DLUserInputDeviceImpl;

#[repr(C)]
#[shared::singleton("FD4::FD4PadManager")]
pub struct FD4PadManager {
    _unk00: [u8; 0x17],
    pub unk18: [OwnedPtr<FD4PadManager0x18>; 4],
}

#[repr(C)]
pub struct FD4PadManager0x18 {
    _vftable: usize,
    pub devices: [OwnedPtr<DLUserInputDeviceImpl>; 4],
}
