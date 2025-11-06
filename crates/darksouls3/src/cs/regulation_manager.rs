use std::ffi::{c_char, c_str::CStr, c_void};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::{mem, num::NonZero, ptr::NonNull, slice};

use cxx_stl::vec::msvc2012::CxxVec;
use shared::{util::IncompleteArrayField, OwnedPtr};

use crate::dltx::{DLString, DLUTF8StringKind};
use crate::param::ParamDef;

#[repr(C)]
#[shared::singleton("CSRegulationManager")]
pub struct CSRegulationManager {
    _vftable: usize,
    _unk8: [u8; 0x8],
    pub params: CxxVec<ParamResCap>,
}

impl CSRegulationManager {
    /// Returns the parameter table for [T].
    pub fn get_param<T: ParamDef>(&self) -> &Parameter<T> {
        let table = &self.params[T::INDEX].param.table;
        table.as_param().unwrap_or_else(|| {
            panic!(
                "Expected param index {} to be {}, was {}",
                T::INDEX,
                T::NAME,
                table.name()
            )
        })
    }

    /// Returns the mutable parameter table for [T].
    pub fn get_mut_param<T: ParamDef>(&mut self) -> &mut Parameter<T> {
        let mut table = &mut self.params[T::INDEX].param.table;
        table
            .as_mut_param()
            // The borrow checker won't let us include the actual name ere
            .unwrap_or_else(|| panic!("Expected param index {} to be {}", T::INDEX, T::NAME))
    }
}

#[repr(C)]
pub struct ParamResCap {
    _vftable: usize,
    _unk8: [u8; 0x8],

    /// The camel-case name of the parameter.
    pub name: DLString<DLUTF8StringKind>,

    _unk40: [u8; 0x28],
    pub param: OwnedPtr<FD4ParamResCap>,
}

#[repr(C)]
pub struct FD4ParamResCap {
    _vftable: usize,
    _unk8: [u8; 0x8],

    /// The camel-case name of the parameter.
    pub name: DLString<DLUTF8StringKind>,

    _unk40: [u8; 0x20],

    /// The total size of [table] in bytes.
    pub table_size: usize,

    pub table: OwnedPtr<ParamTable>,
}

#[repr(C)]
pub struct ParamTable {
    _unk0: [u8; 0xa],
    pub length: u16,
    _unkc: [u8; 0x4],

    /// The offset of the parameter's snake-case name from the beginning of this
    /// struct.
    pub name_offset: usize,

    _unk18: [u8; 0x28],

    row_info: IncompleteArrayField<ParamRowInfo>,
    // Note: After the row_info is an incomplete array of the actual parameter
    // data.
}

impl ParamTable {
    /// The parameter's snake-case name.
    ///
    /// ## Panic
    ///
    /// Panics if this string isn't valid UTF-8.
    pub fn name(&self) -> &str {
        let name_ptr = (&raw const self)
            .map_addr(|addr| addr + self.name_offset)
            .cast::<c_char>();
        // Safety: We trust the game's memory layout.
        unsafe { CStr::from_ptr(name_ptr) }.to_str().unwrap()
    }

    /// Returns a pointer to the beginning of the section of the table that
    /// contains the actual parameter data.
    pub fn data(&self) -> NonNull<c_void> {
        let offset = (self.length as usize) * mem::size_of::<ParamRowInfo>();
        NonNull::from_ref(self)
            .map_addr(|addr| addr.saturating_add(offset))
            .cast::<c_void>()
    }

    /// Returns the header information about each row as a slice.
    pub fn row_info(&self) -> &[ParamRowInfo] {
        // Safety: We trust the game to report lengths accurately.
        unsafe { self.row_info.as_slice(self.length.try_into().unwrap()) }
    }

    /// If [name] matches [T]'s [ParamDef::NAME], converts this to a [Parameter].
    pub fn as_param<T: ParamDef>(&self) -> Option<&Parameter<T>> {
        if self.name() == T::NAME {
            // Safety: [Parameter] is a transparent wrapper around [ParamTable].
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }

    /// If [name] matches [T]'s [ParamDef::NAME], converts this to a mutable
    /// [Parameter].
    pub fn as_mut_param<T: ParamDef>(&mut self) -> Option<&mut Parameter<T>> {
        if self.name() == T::NAME {
            // Safety: [Parameter] is a transparent wrapper around [ParamTable].
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct ParamRowInfo {
    /// The ID of the parameter row this describes.
    pub id: u64,

    /// The offset (in bytes) from the beginning of the [ParamTable] that
    /// contains this to the data for the parameter this represents.
    pub offset: usize,

    _unk10: u64,
}

/// A safe and usable view of a single parameter table, associated with a
/// particular parameter type.
#[repr(transparent)]
pub struct Parameter<T: ParamDef> {
    pub table: ParamTable,
    _phantom: PhantomData<T>,
}

impl<T: ParamDef> Parameter<T> {
    /// Returns a slice of all the rows in this parameter.
    ///
    /// Note that these **do not** contain the row indexes. For that, you must
    /// use [iter].
    pub fn as_slice(&self) -> &[T] {
        // Safety: We trust the game to report lengths accurately.
        unsafe {
            slice::from_raw_parts(
                self.table.data().cast().as_ptr(),
                self.table.length.try_into().unwrap(),
            )
        }
    }

    /// Returns a mutable slice of all the rows in this parameter.
    ///
    /// Note that these **do not** contain the row indexes. For that, you must
    /// use [iter].
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // Safety: We trust the game to report lengths accurately.
        unsafe {
            slice::from_raw_parts_mut(
                self.table.data().cast().as_ptr(),
                self.table.length.try_into().unwrap(),
            )
        }
    }

    /// If this parameter has a row with the given [id], returns it. Otherwise
    /// returns None.
    pub fn get(&self, id: u64) -> Option<&T> {
        // Safety: We trust DS3's memory layout
        Some(unsafe { self.ptr_for_id(id)?.as_ref() })
    }

    /// If this parameter has a row with the given [id], returns a mutable
    /// reference to it. Otherwise returns None.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut T> {
        // Safety: We trust DS3's memory layout
        Some(unsafe { self.ptr_for_id(id)?.as_mut() })
    }

    /// Returns the pointer to the row with the given [id], or null if no such
    /// row exists.
    fn ptr_for_id(&self, id: u64) -> Option<NonNull<T>> {
        let infos = self.table.row_info();
        let index = infos.binary_search_by_key(&id, |info| info.id).ok()?;
        Some(
            NonNull::from_ref(&self.table)
                .map_addr(|addr| addr.saturating_add(infos[index].offset))
                .cast(),
        )
    }

    /// Returns an iterator that emits `(id, row)` pairs for each row in this
    /// parameter.
    pub fn iter(&self) -> ParamIter<'_, T> {
        ParamIter {
            param: self,
            inner: self.table.row_info().iter(),
        }
    }

    /// Returns an iterator that emits mutable `(id, row)` pairs for each row in
    /// this parameter.
    pub fn iter_mut(&mut self) -> ParamIterMut<'_, T> {
        ParamIterMut {
            param: self,
            inner: self.table.row_info().iter(),
        }
    }
}

impl<T: ParamDef> Index<u64> for Parameter<T> {
    type Output = T;

    fn index(&self, index: u64) -> &T {
        self.get(index).expect("no row found for ID")
    }
}

impl<T: ParamDef> IndexMut<u64> for Parameter<T> {
    fn index_mut(&mut self, index: u64) -> &mut T {
        self.get_mut(index).expect("no row found for ID")
    }
}

/// An iterator over parameters and their IDs.
pub struct ParamIter<'a, T: ParamDef> {
    param: &'a Parameter<T>,
    inner: slice::Iter<'a, ParamRowInfo>,
}

impl<'a, T: ParamDef> Iterator for ParamIter<'a, T> {
    type Item = (u64, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let info = self.inner.next()?;
        let ptr = NonNull::from_ref(&self.param.table)
            .map_addr(|addr| addr.saturating_add(info.offset))
            .cast();
        // Safety: We trust DS3's memory layout.
        unsafe { Some((info.id, ptr.as_ref())) }
    }
}

/// An iterator over mutable parameters and their IDs.
pub struct ParamIterMut<'a, T: ParamDef> {
    param: &'a Parameter<T>,
    inner: slice::Iter<'a, ParamRowInfo>,
}

impl<'a, T: ParamDef> Iterator for ParamIterMut<'a, T> {
    type Item = (u64, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let info = self.inner.next()?;
        let mut ptr = NonNull::from_ref(&self.param.table)
            .map_addr(|addr| addr.saturating_add(info.offset))
            .cast();
        // Safety: We trust DS3's memory layout.
        unsafe { Some((info.id, ptr.as_mut())) }
    }
}
