use vtable_rs::VPtr;

use crate::dlui::DynamicBitset;

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
