use std::{mem::MaybeUninit, ptr::NonNull};

use super::WorldRes;
use shared::{FromStatic, InstanceError, InstanceResult, Program};

pub struct FieldArea {
    _vftable: usize,

    pub world_res: Option<NonNull<WorldRes>>,

    world_res_2: Option<NonNull<WorldRes>>, // Always the same as [world_res], apparently

    _game_rend: u64,
    _unk20: u32,
    _chr_cam: u64,
    _unk30: [u8; 0x30],
    _hit_ins: u64,
    _unk68: u64,
    _field_backread: usize,
    _unk78: [u8; 0x60],
    _self: NonNull<FieldArea>,
    _unke0: usize,
    _unke8: [u8; 8],
}

impl FieldArea {
    pub fn world_res(&self) -> Option<&WorldRes> {
        self.world_res.map(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn world_res_mut(&mut self) -> Option<&mut WorldRes> {
        self.world_res.map(|mut ptr| unsafe { ptr.as_mut() })
    }
}

impl FromStatic for FieldArea {
    unsafe fn instance() -> InstanceResult<&'static mut Self> {
        use crate::rva;
        use pelite::pe64::Pe;

        let target = *(Program::current()
            .rva_to_va(rva::get().field_area_ptr)
            .map_err(|_| InstanceError::NotFound)? as *const *mut Self);

        unsafe { target.as_mut().ok_or(InstanceError::Null) }
    }
}
