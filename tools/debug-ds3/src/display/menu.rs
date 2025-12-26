use hudhook::imgui::Ui;

use darksouls3::sprj::*;

use super::StatefulDebugDisplay;

#[derive(Default)]
pub struct ItemGetMenuManDebugState {
    item_id: String,
    quantity: String,
    unk14: bool,
}

impl StatefulDebugDisplay for ItemGetMenuMan {
    type State = ItemGetMenuManDebugState;

    fn render_debug(&mut self, ui: &&mut Ui, state: &mut Self::State) {
        {
            let _tok = ui.push_item_width(150.);
            ui.input_text("Item ID ", &mut state.item_id).build();
        }

        ui.same_line();
        {
            let _tok = ui.push_item_width(100.);
            ui.input_text("Quantity", &mut state.quantity).build();
        }

        ui.checkbox("In Box", &mut state.unk14);

        let item_id = state
            .item_id
            .parse::<u32>()
            .ok()
            .and_then(|i| ItemId::try_from(i).ok());

        let quantity = state.quantity.parse::<u32>();

        ui.same_line_with_pos(ui.window_content_region_max()[0] - 200.);
        {
            let _tok = ui.begin_enabled(item_id.is_some() && quantity.is_ok());
            if ui.button("Show Popup") {
                self.show_item(item_id.unwrap(), quantity.unwrap(), state.unk14);
            }
        }
    }
}
