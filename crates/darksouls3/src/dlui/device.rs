use cxx_stl::vec::msvc2012::CxxVec;

use crate::dlkr::{DLAllocatorRef, DLPlainLightMutex};
use crate::dlui::DynamicBitset;

#[repr(C)]
pub struct DLUserInputDevice {
    _vftable: usize,
    pub allocator: DLAllocatorRef,
    _unk10: [u8; 0x50],
    pub extensions: CxxVec<usize>,
}

#[repr(C)]
pub struct DLUserInputDeviceImpl {
    pub device: DLUserInputDevice,
    _unk80: [u8; 0x10],
    pub mutex: DLPlainLightMutex,
    _unkc8: [u8; 0x8],
    _key_info1: VirtualAnalogKeyInfo,
    _key_info2: VirtualAnalogKeyInfo,
    _unk120: CxxVec<u64>,
    pub input_data: VirtualInputData,
}

#[repr(C)]
struct VirtualAnalogKeyInfo {
    _vftable: usize,
    _unk08: CxxVec<u64>,
}

#[repr(C)]
pub struct VirtualInputData {
    _vftable: usize,
    _key_info: VirtualAnalogKeyInfo,
    pub bitset: DynamicBitset,
}
