use std::ptr::NonNull;

use cxx_stl::vec::msvc2012::CxxVec;
use shared::OwnedPtr;

use crate::sprj::CategorizedItemID;

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
    _unk08: [u8; 0x1a0],
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
    pub item_entries_count: i32,
    pub is_inventory_full: bool,
    _unk8d: [u8; 0x13],
}

#[repr(C)]
pub struct InventoryItemListAccessor {
    pub head: NonNull<EquipInventoryDataListEntry>,
    pub count: NonNull<u32>,
}

#[repr(C)]
pub struct InventoryItemsData {
    _unk00: u32,

    /// Capacity of the normal items inventory.
    pub normal_items_capacity: u32,

    /// Pointer to the head of the normal items inventory.
    pub normal_items_head: OwnedPtr<EquipInventoryDataListEntry>,

    /// Count of the items in the normal items inventory.
    pub normal_items_count: u32,

    /// Capacity of the key items inventory.
    pub key_items_capacity: u32,

    /// Pointer to the head of the key items inventory.
    pub key_items_head: OwnedPtr<EquipInventoryDataListEntry>,

    /// Count of the items in the key items inventory.
    pub key_items_count: u32,

    _unk24: [u8; 0x14],

    /// Pointers to the active normal item list and its count. All inventory
    /// reads and writes in the game will go through this.
    pub normal_items_accessor: InventoryItemListAccessor,

    /// Pointers to the active key item list and its count. All inventory reads
    /// and writes in the game will go through this.
    pub key_items_accessor: InventoryItemListAccessor,

    /// Contains the indices into the item ID mapping list.
    pub item_id_mapping_indices: OwnedPtr<[u16; 2017]>,

    _unk60: u64,
    _item_id_mapping: usize,
    _next_index: u16,
    _unk72: [u8; 0x6],
}

impl InventoryItemsData {
    /// Returns the non-key items in the player's inventory as a slice.
    pub fn normal_items(&self) -> &[EquipInventoryDataListEntry] {
        unsafe {
            std::slice::from_raw_parts(
                self.normal_items_head.as_ptr(),
                self.normal_items_count as usize,
            )
        }
    }

    /// Returns the non-key items in the player's inventory as a mutable slice.
    pub fn normal_items_mut(&mut self) -> &mut [EquipInventoryDataListEntry] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.normal_items_head.as_ptr(),
                self.normal_items_count as usize,
            )
        }
    }

    /// Returns whether the player's non-key item inventory is full.
    pub fn is_normal_items_full(&self) -> bool {
        self.normal_items_count >= self.normal_items_capacity
    }

    /// Returns the (non-multiplayer) key items in the player's inventory as a
    /// slice.
    pub fn key_items(&self) -> &[EquipInventoryDataListEntry] {
        unsafe {
            std::slice::from_raw_parts(self.key_items_head.as_ptr(), self.key_items_count as usize)
        }
    }

    /// Returns the (non-multiplayer) key items in the player's inventory as a
    /// mutable slice.
    pub fn key_items_mut(&mut self) -> &mut [EquipInventoryDataListEntry] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.key_items_head.as_ptr(),
                self.key_items_count as usize,
            )
        }
    }

    /// Returns whether the player's (non-multiplayer) key item inventory is full.
    pub fn is_key_items_full(&self) -> bool {
        self.key_items_count >= self.key_items_capacity
    }
}

#[repr(C)]
pub struct EquipInventoryDataListEntry {
    /// Handle to the gaitem instance which describes additional properties of
    /// the inventory item, like durability.
    pub gaitem_handle: u32,

    /// The ID of the item in this inventory slot.
    pub item_id: CategorizedItemID,

    /// Quantity of the item we have.
    pub quantity: u64,
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
        assert_eq!(0x118, size_of::<FaceData>());
        assert_eq!(0x78, size_of::<InventoryItemsData>());
        assert_eq!(0xa0, size_of::<EquipInventoryData>());
        assert_eq!(0x328, size_of::<EquipGameData>());
        assert_eq!(0x140, size_of::<PlayerInfo>());
        assert_eq!(0x950, size_of::<PlayerGameData>());
    }
}
