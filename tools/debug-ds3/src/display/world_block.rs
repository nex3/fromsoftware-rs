use hudhook::imgui::*;

use darksouls3::sprj::*;

use super::DebugDisplay;

impl DebugDisplay for WorldBlockChr {
    fn render_debug(&mut self, ui: &&mut Ui) {
        self.chr_set.render_debug(ui);

        if ui.collapsing_header("Mappings", TreeNodeFlags::empty()) {
            ui.indent();
            if let Some(_t) = ui.begin_table_header_with_flags(
                "world-block-chr-mappings",
                [
                    TableColumnSetup::new("Entity ID"),
                    TableColumnSetup::new("FieldIns Type"),
                    TableColumnSetup::new("Container"),
                    TableColumnSetup::new("Index"),
                ],
                TableFlags::RESIZABLE | TableFlags::SIZING_FIXED_FIT,
            ) {
                for mapping in self.mappings() {
                    ui.table_next_column();
                    ui.text(format!("{}", mapping.entity_id));

                    ui.table_next_column();
                    ui.text(format!("{:?}", mapping.selector.field_ins_type()));

                    ui.table_next_column();
                    ui.text(format!("0x{:x}", mapping.selector.container()));

                    ui.table_next_column();
                    ui.text(format!("0x{:x}", mapping.selector.index()));
                }
            }
            ui.unindent();
        }
    }
}
