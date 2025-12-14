use darksouls3::sprj::*;
use hudhook::imgui::{TableColumnSetup, TableFlags, TreeNodeFlags, Ui};

use super::DebugDisplay;

impl DebugDisplay for PlayerIns {
    fn render_debug(&mut self, ui: &&mut Ui) {
        self.super_chr_ins.render_debug(ui);

        if ui.collapsing_header("PlayerGameData", TreeNodeFlags::empty()) {
            ui.indent();
            unsafe { self.player_game_data.as_mut() }.render_debug(ui);
            ui.unindent();
        }
    }
}

impl DebugDisplay for PlayerGameData {
    fn render_debug(&mut self, ui: &&mut Ui) {
        self.player_info.render_debug(ui);

        if ui.collapsing_header("EquipGameData", TreeNodeFlags::empty()) {
            ui.indent();
            self.equipment.render_debug(ui);
            ui.unindent();
        }

        if let Some(storage) = &mut self.storage
            && ui.collapsing_header("Storage Box", TreeNodeFlags::empty())
        {
            ui.indent();
            storage.render_debug(ui);
            ui.unindent();
        }
    }
}

impl DebugDisplay for PlayerInfo {
    fn render_debug(&mut self, ui: &&mut Ui) {
        ui.text(format!("ID: {}", self.id));
        if !self.name().is_empty() {
            ui.text(format!("Name: {}", self.name()));
        }
        ui.text(format!("Vigor: {}", self.vigor));
        ui.text(format!("Attunement: {}", self.attunement));
        ui.text(format!("Endurance: {}", self.endurance));
        ui.text(format!("Vitality: {}", self.vitality));
        ui.text(format!("Strength: {}", self.strength));
        ui.text(format!("Dexterity: {}", self.dexterity));
        ui.text(format!("Intelligence: {}", self.intelligence));
        ui.text(format!("Faith: {}", self.faith));
        ui.text(format!("Luck: {}", self.luck));
    }
}

impl DebugDisplay for EquipGameData {
    fn render_debug(&mut self, ui: &&mut Ui) {
        if ui.collapsing_header("EquipInventoryData", TreeNodeFlags::empty()) {
            ui.indent();
            self.equip_inventory_data.render_debug(ui);
            ui.unindent();
        }
    }
}

impl DebugDisplay for EquipInventoryData {
    fn render_debug(&mut self, ui: &&mut Ui) {
        let label = format!(
            "Items ({}/{})",
            self.items_data.items_len(),
            self.items_data.total_capacity
        );
        if ui.collapsing_header(label.as_str(), TreeNodeFlags::empty()) {
            ui.indent();
            if let Some(_t) = ui.begin_table_header_with_flags(
                "equip-inventory-data-items",
                [
                    TableColumnSetup::new("Index"),
                    TableColumnSetup::new("Gaitem Handle"),
                    TableColumnSetup::new("Item ID"),
                    TableColumnSetup::new("Quantity"),
                ],
                TableFlags::RESIZABLE | TableFlags::SIZING_FIXED_FIT,
            ) {
                self.items_data
                    .items()
                    .enumerate()
                    .for_each(|(index, item)| {
                        ui.table_next_column();
                        ui.text(index.to_string());

                        ui.table_next_column();
                        ui.text(item.gaitem_handle.to_string());

                        ui.table_next_column();
                        ui.text(format!("{:?}", item.item_id));

                        ui.table_next_column();
                        ui.text(item.quantity.to_string());
                    });
            }
            ui.unindent();
        }
    }
}

impl DebugDisplay for ChrIns {
    fn render_debug(&mut self, ui: &&mut Ui) {
        if ui.button("Kill") {
            self.kill();
        }

        let data = &self.modules.data;
        ui.text(format!("HP: {}/{}", data.hp, data.max_hp));
        ui.text(format!("MP: {}/{}", data.fp, data.max_fp));
        ui.text(format!("Stamina: {}/{}", data.stamina, data.max_stamina));
    }
}
