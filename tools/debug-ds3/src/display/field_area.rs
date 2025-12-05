use hudhook::imgui::{TreeNodeFlags, Ui};

use darksouls3::sprj::*;

use super::DebugDisplay;

impl DebugDisplay for FieldArea {
    fn render_debug(&mut self, ui: &&mut Ui) {
        if let Some(world_res) = self.world_res_mut() {
            world_res.super_world_info.render_debug(ui);
        } else {
            ui.text("World res: null");
        }
    }
}

impl DebugDisplay for WorldInfo {
    fn render_debug(&mut self, ui: &&mut Ui) {
        if ui.collapsing_header(
            format!("Area infos: {} ##{:p}", self.area_info().len(), self),
            TreeNodeFlags::empty(),
        ) {
            ui.indent();
            for area_info in self.area_info_mut() {
                if ui.collapsing_header(
                    format!("Area {} ##{:p}", area_info.area_number, area_info),
                    TreeNodeFlags::empty(),
                ) {
                    ui.indent();
                    area_info.render_debug(ui);
                    ui.unindent();
                }
            }
            ui.unindent();
        }
    }
}

impl DebugDisplay for WorldAreaInfo {
    fn render_debug(&mut self, ui: &&mut Ui) {
        for block in self.block_info() {
            ui.text(format!(
                "Block {}: event index {}",
                block.block_id.group(),
                block.world_block_index
            ));
        }
    }
}
