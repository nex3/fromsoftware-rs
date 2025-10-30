use std::mem::MaybeUninit;
use std::sync::LazyLock;

use pelite::pe64::Pe;
use shared::Program;
use vtable_rs::VPtr;

use crate::dlkr::DLAllocatorRef;
use crate::dlui::DynamicBitset;

static DL_USER_INPUT_SUPPRESSOR_CONSTRUCTOR_VA: LazyLock<u64> =
    LazyLock::new(|| Program::current().rva_to_va(0x17daa20).unwrap());

#[vtable_rs::vtable]
pub trait DLUserInputSuppressorVmt {
    fn unk00(&self);
    fn destructor(&mut self, param_2: bool);
}

#[repr(C)]
pub struct DLUserInputSuppressor {
    pub vftable: VPtr<dyn DLUserInputSuppressorVmt, Self>,
    pub bitset1: DynamicBitset,
    pub bitset2: DynamicBitset,
}

impl DLUserInputSuppressor {
    pub fn new(allocator: DLAllocatorRef) -> Self {
        let mut result = MaybeUninit::<DLUserInputSuppressor>::uninit();
        let constructor: extern "win64" fn(
            &mut MaybeUninit<DLUserInputSuppressor>,
            DLAllocatorRef,
        ) -> usize = unsafe { std::mem::transmute(*DL_USER_INPUT_SUPPRESSOR_CONSTRUCTOR_VA) };
        constructor(&mut result, allocator);

        // Safety: The constructor initializes the struct.
        unsafe { result.assume_init() }
    }
}

impl Drop for DLUserInputSuppressor {
    fn drop(&mut self) {
        (self.vftable.destructor)(self, false);
    }
}
