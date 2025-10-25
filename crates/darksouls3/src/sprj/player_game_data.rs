use std::ptr::NonNull;

use shared::OwnedPtr;

use crate::sprj::CategorizedItemID;

#[repr(C)]
pub struct EquipGameData {
    _vftable: usize,
    _unk08: [u8; 0x1a0],
    pub equip_inventory_data: EquipInventoryData,
}

#[repr(C)]
pub struct EquipInventoryData {
    _vftable: usize,
    _unk08: u64,
    pub items_data: InventoryItemsData,
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
