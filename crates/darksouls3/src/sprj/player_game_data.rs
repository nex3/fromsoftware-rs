use std::marker::PhantomData;
use std::mem;
use std::num::NonZero;
use std::ops::{Index, IndexMut};
use std::ptr::NonNull;
use std::slice;
use std::{iter, iter::FusedIterator};

use bitfield::bitfield;
use cxx_stl::vec::msvc2012::CxxVec;
use shared::OwnedPtr;

use crate::sprj::{CategorizedItemID, MaybeInvalidCategorizedItemID};

#[repr(C)]
/// Source of name: RTTI
pub struct PlayerGameData {
    _vftable: usize,
    _unk08: u64,
    pub player_info: PlayerInfo,
    _unk150: [u8; 0xD8],
    pub equipment: EquipGameData,
    _unk550: [u8; 0x148],

    pub face_data: FaceData,

    /// The contents of the storage box.
    pub storage: OwnedPtr<EquipInventoryData>,

    _gesture_data: usize,
    _unk7c0: [u8; 0x58],
    _unk810: CxxVec<u64>,
    _unk830: [u8; 0xf0],
    _menu_ref_special_effect_1: usize,
    _menu_ref_special_effect_2: usize,
    _unk930: [u8; 0x20],
}

#[repr(C)]
/// Source of name: chosen by us
pub struct PlayerInfo {
    _unk00: [u8; 0x78],

    /// The character's name, in UTF-16. The final word is always 0, to ensure
    /// the string is null-terminated.
    pub character_name: [u16; 17],

    _unk9a: [u8; 0xa6],
}

impl PlayerInfo {
    /// Returns the player's name.
    pub fn name(&self) -> String {
        let length = self
            .character_name
            .iter()
            .position(|c| *c == 0)
            .unwrap_or(self.character_name.len());
        String::from_utf16(&self.character_name[..length]).unwrap()
    }
}

#[repr(C)]
pub struct EquipGameData {
    _vftable: usize,
    _unk08: [u8; 0x1c],

    /// A mapping from equipment slots to the [EquipInventoryData] indices of
    /// items currently in those slots.
    pub equipment_indexes: [i32; 22],

    _unk7c: [u8; 0x12C],
    pub equip_inventory_data: EquipInventoryData,
    _unk248: [u8; 0xe0],
}

impl EquipGameData {
    /// For whatever reason, DS3 has an EquipGameData active on the main menu as
    /// well as in the context of an individual game. This function returns
    /// whether a given instance is for the synthetic loading screen character
    /// rather than a real loaded world.
    pub fn is_main_menu(&self) -> bool {
        // For some even stranger reason, the loading screen save actually does
        // have a handful of items. However, even a totally fresh save has more
        // items than that, so we check for 12 items which is exactly how many
        // the loading screen has. This could be tricked if a player discarded
        // all their starting equipment, so... don't do that.
        return self.equip_inventory_data.items_data.normal_items_count == 12;
    }
}

#[repr(C)]
pub struct EquipInventoryData {
    _vftable: usize,
    _unk08: u64,
    pub items_data: InventoryItemsData,

    /// The largest [EquipInventoryData] index that *might* not be empty.
    /// There's no guarantee that this isn't actually empty.
    pub max_item_index: i32,

    pub is_inventory_full: bool,
    _unk8d: [u8; 0x13],
}

#[repr(C)]
pub struct InventoryItemListAccessor {
    pub head: NonNull<EquipInventoryDataListEntry>,
    pub len: NonNull<u32>,
}

#[repr(C)]
pub struct InventoryItemsData {
    /// The total number of items the player can hold.
    pub total_capacity: u32,

    /// Capacity of the [normal_items_head] array.
    pub normal_items_capacity: u32,

    /// Pointer to the head of the normal items inventory.
    ///
    /// **Note:** This array is not dense. If an entry in the middle is emptied
    /// due to an item being removed from the player's inventory, other items
    /// are *not* rearranged to fill the hole.
    pub normal_items_head: OwnedPtr<EquipInventoryDataListEntry>,

    /// The number of normal items in the inventory.
    pub normal_items_count: u32,

    /// Capacity of the [key_items_head] array.
    pub key_items_capacity: u32,

    /// Pointer to the head of the key items inventory.
    ///
    /// **Note:** This array is not dense. If an entry in the middle is emptied
    /// due to an item being removed from the player's inventory, other items
    /// are *not* rearranged to fill the hole.
    pub key_items_head: OwnedPtr<EquipInventoryDataListEntry>,

    /// The number of key items in the inventory.
    pub key_items_count: u32,

    _unk24: [u8; 0x14],

    /// Pointers to the active normal item list and its count. All inventory
    /// reads and writes in the game will go through this.
    pub normal_items_accessor: InventoryItemListAccessor,

    /// Pointers to the active key item list and its count. All inventory reads
    /// and writes in the game will go through this.
    pub key_items_accessor: InventoryItemListAccessor,

    /// A map from item IDs (mod 2017) to the index of their mapping linked list
    /// in [item_id_mappings].
    ///
    /// This is populated as items are added to the inventory. All entries begin
    /// as -1.
    pub item_id_mapping_indices: OwnedPtr<[i16; 2017]>,

    _unk60: u64,

    /// A [total_capacity]-length array of mappings from item IDs to indices in
    /// [normal_items_head] or [key_items_head].
    ///
    /// This is iteslf indexed by [item_id_mapping_indices].
    pub item_id_mappings: OwnedPtr<ItemIdMapping>,

    /// The index into [item_id_mappings] that should be used next time an item
    /// is added to the inventory whose index (mod 2017) hasn't yet been allocated
    /// to [item_id_mapping_indices].
    pub next_index: u16,

    _unk72: [u8; 0x6],
}

impl InventoryItemsData {
    /// Returns an iterator over all the non-empty entries in the player's
    /// inventory.
    ///
    /// This iterates over key items first, followed by normal items.
    pub fn items(&self) -> ItemsIterator<'_> {
        ItemsIterator(
            self.key_entries()
                .iter()
                .chain(self.normal_entries().iter()),
        )
    }

    /// Returns an iterator over all the mutable non-empty entries in the
    /// player's inventory.
    ///
    /// This iterates over key items first, followed by normal items.
    pub fn items_mut(&self) -> ItemsIteratorMut<'_> {
        ItemsIteratorMut(
            unsafe {
                std::slice::from_raw_parts_mut(
                    self.key_items_head.as_ptr(),
                    self.key_items_capacity as usize,
                )
            }
            .iter_mut()
            .chain(
                unsafe {
                    std::slice::from_raw_parts_mut(
                        self.normal_items_head.as_ptr(),
                        self.normal_items_capacity as usize,
                    )
                }
                .iter_mut(),
            ),
        )
    }

    /// Returns a slice over all the [EquipInventoryDataListEntry] allocated for
    /// this [InventoryItemsData], whether or not they're empty or in range of
    /// [key_items_len].
    pub fn key_entries(&self) -> &[EquipInventoryDataListEntry] {
        unsafe {
            std::slice::from_raw_parts(
                self.key_items_head.as_ptr(),
                self.key_items_capacity as usize,
            )
        }
    }

    /// Returns a mutable slice over all the [EquipInventoryDataListEntry]
    /// allocated for this [InventoryItemsData], whether or not they're empty or
    /// in range of [key_items_len].
    pub fn key_entries_mut(&mut self) -> &mut [EquipInventoryDataListEntry] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.key_items_head.as_ptr(),
                self.key_items_capacity as usize,
            )
        }
    }

    /// Returns a slice over all the [EquipInventoryDataListEntry] allocated for
    /// this [InventoryItemsData], whether or not they're empty or in range of
    /// [normal_items_len].
    pub fn normal_entries(&self) -> &[EquipInventoryDataListEntry] {
        unsafe {
            std::slice::from_raw_parts(
                self.normal_items_head.as_ptr(),
                self.normal_items_capacity as usize,
            )
        }
    }

    /// Returns a mutable slice over all the [EquipInventoryDataListEntry]
    /// allocated for this [InventoryItemsData], whether or not they're empty or
    /// in range of [normal_items_len].
    pub fn normal_entries_mut(&mut self) -> &mut [EquipInventoryDataListEntry] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.normal_items_head.as_ptr(),
                self.normal_items_capacity as usize,
            )
        }
    }
}

impl Index<u32> for InventoryItemsData {
    type Output = EquipInventoryDataListEntry;

    /// Indexes both the key and normal item entries of [InventoryItemsData]
    /// using the same logic as the game.
    ///
    /// If [index] is less than [key_items_capacity], this returns a key items
    /// entry. If it's greater than or equal to [key_items_capacity] but less
    /// than that plus [normal_items_capacity], this returns a normal item
    /// entry. Otherwise, it panics.
    fn index(&self, index: u32) -> &Self::Output {
        if index < self.key_items_capacity {
            return &self.key_entries()[index as usize];
        }

        let index = index - self.key_items_capacity;
        if index < self.normal_items_capacity {
            return &self.normal_entries()[index as usize];
        }

        panic!("index {} out of range", index)
    }
}

impl IndexMut<u32> for InventoryItemsData {
    /// Mutably indexes both the key and normal item entries of
    /// [InventoryItemsData] using the same logic as the game.
    ///
    /// If [index] is less than [key_items_capacity], this returns a key items
    /// entry. If it's greater than or equal to [key_items_capacity] but less
    /// than that plus [normal_items_capacity], this returns a normal item
    /// entry. Otherwise, it panics.
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        if index < self.key_items_capacity {
            return &mut self.key_entries_mut()[index as usize];
        }

        let index = index - self.key_items_capacity;
        if index < self.normal_items_capacity {
            return &mut self.normal_entries_mut()[index as usize];
        }

        panic!("index {} out of range", index)
    }
}

/// An iterator over both normal and key items in [InventoryItemsData] that
/// exposes only non-empty entries.
///
/// Returned by [InventoryItemsData.items].
pub struct ItemsIterator<'a>(
    iter::Chain<
        slice::Iter<'a, EquipInventoryDataListEntry>,
        slice::Iter<'a, EquipInventoryDataListEntry>,
    >,
);

impl<'a> Iterator for ItemsIterator<'a> {
    type Item = &'a NonEmptyEquipInventoryDataListEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                Some(entry) => match entry.as_non_empty() {
                    Some(entry) => return Some(entry),
                    None => {}
                },
                None => return None,
            }
        }
    }
}

impl<'a> FusedIterator for ItemsIterator<'a> {}

/// A mutable iterator over both normal and key items in [InventoryItemsData]
/// that exposes only non-empty entries.
///
/// Returned by [InventoryItemsData.items_mut].
pub struct ItemsIteratorMut<'a>(
    iter::Chain<
        slice::IterMut<'a, EquipInventoryDataListEntry>,
        slice::IterMut<'a, EquipInventoryDataListEntry>,
    >,
);

impl<'a> Iterator for ItemsIteratorMut<'a> {
    type Item = &'a mut NonEmptyEquipInventoryDataListEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                Some(entry) => match entry.as_non_empty_mut() {
                    Some(entry) => return Some(entry),
                    None => {}
                },
                None => return None,
            }
        }
    }
}

impl<'a> FusedIterator for ItemsIteratorMut<'a> {}

/// An entry in [InventoryItemsData] that may be empty.
///
/// This is empty if and only if [item_id] is
/// [MaybeInvalidCategorizedItemID::INVALID]. An empty entry doesn't currently
/// represent an item in the player's inventory, but may in the future. Its
/// [gaitem_handle] and [quantity] are generally both 0.
#[repr(C)]
pub struct EquipInventoryDataListEntry {
    /// Handle to the gaitem instance which describes additional properties of
    /// the inventory item, like durability.
    ///
    /// This is always zero if this entry is empty.
    pub gaitem_handle: u32,

    /// The raw ID of the item in this inventory slot. This is invalid if the
    /// inventory item has since been removed.
    ///
    /// This is always [MaybeInvalidCategorizedItemID::INVALID] if this entry is
    /// empty.
    pub item_id: MaybeInvalidCategorizedItemID,

    /// Quantity of the item we have.
    ///
    /// This is always zero if this entry is empty.
    pub quantity: u32,

    _unk0c: [u8; 4],
}

impl EquipInventoryDataListEntry {
    /// Returns whether this entry is empty.
    pub fn is_empty(&self) -> bool {
        !self.item_id.is_valid()
    }

    /// If this isn't empty, returns it as a
    /// [NonEmptyEquipInventoryDataListEntry]. Otherwise, returns `None`.
    pub fn as_non_empty(&self) -> Option<&NonEmptyEquipInventoryDataListEntry> {
        if !self.is_empty() {
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }

    /// If this isn't empty, returns it as a mutable
    /// [NonEmptyEquipInventoryDataListEntry]. Otherwise, returns `None`.
    pub fn as_non_empty_mut(&mut self) -> Option<&mut NonEmptyEquipInventoryDataListEntry> {
        if !self.is_empty() {
            Some(unsafe { mem::transmute(self) })
        } else {
            None
        }
    }
}

/// An [EquipInventoryDataListEntry] that's been verified to be non-empty, and
/// so whose fields are all valid.
#[repr(C)]
pub struct NonEmptyEquipInventoryDataListEntry {
    /// Handle to the gaitem instance which describes additional properties of
    /// the inventory item, like durability.
    pub gaitem_handle: NonZero<u32>,

    /// The raw ID of the item in this inventory slot. This is invalid if the
    /// inventory item has since been removed.
    pub item_id: CategorizedItemID,

    /// Quantity of the item we have.
    pub quantity: u32,

    _unk0c: [u8; 4],
}

impl AsRef<EquipInventoryDataListEntry> for NonEmptyEquipInventoryDataListEntry {
    fn as_ref(&self) -> &EquipInventoryDataListEntry {
        // Safety: All valid NonEmptyEquipInventoryDataListEntries are also
        // EquipInventoryDataListEntries.
        unsafe { mem::transmute(self) }
    }
}

impl AsMut<EquipInventoryDataListEntry> for NonEmptyEquipInventoryDataListEntry {
    fn as_mut(&mut self) -> &mut EquipInventoryDataListEntry {
        // Safety: All valid NonEmptyEquipInventoryDataListEntries are also
        // EquipInventoryDataListEntries.
        unsafe { mem::transmute(self) }
    }
}

#[repr(C)]
pub struct ItemIdMapping {
    /// The ID of the item whose mapping this represents. This is invalid if
    /// there aren't currently any items in this bucket.
    item_id: MaybeInvalidCategorizedItemID,

    /// Indices into [InventoryItemsData]'s lists related to this mapping.
    indices: ItemIdMappingIndices,
}

bitfield! {
    pub struct ItemIdMappingIndices(u32);
    impl Debug;

    /// The index in [InventoryItemsData] at which [item_id_raw] appears.
    ///
    /// If this is less than [InventoryItemsData.key_items_capacity], it's a
    /// direct index into [InventoryItemsData.key_items_head]. Otherwise, this
    /// minus [InventoryItemsData.key_items_capacity] is an index into
    /// [InventoryItemsData.normal_items_head].
    pub u16, inventory_index, set_inventory_index: 12, 0;

    /// If [ItemIdMapping.item_id_raw] is invalid, this is one plus the index
    /// into [InventoryItemsData.item_id_mappings] that should be used for the
    /// next new mapping after this one has been allocated.
    ///
    /// If [ItemIdMapping.item_id_raw] is valid and there are additional items
    /// in the same bucket as this one, this is one plus the index into
    /// [InventoryItemsData.item_id_mappings] for the next item in the bucket.
    ///
    /// Otherwise, this is 0.
    pub u16, next_index, set_next_index: 24, 12;

    unk04_25, _: 25;
}

#[repr(C)]
pub struct FaceData {
    _vftable: usize,
    _unk08: [u8; 0x108],
    pub player_game_data: *mut PlayerGameData,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn proper_sizes() {
        assert_eq!(0x10, size_of::<EquipInventoryDataListEntry>());
        assert_eq!(0x118, size_of::<FaceData>());
        assert_eq!(0x78, size_of::<InventoryItemsData>());
        assert_eq!(0xa0, size_of::<EquipInventoryData>());
        assert_eq!(0x328, size_of::<EquipGameData>());
        assert_eq!(0x140, size_of::<PlayerInfo>());
        assert_eq!(0x950, size_of::<PlayerGameData>());
    }
}
