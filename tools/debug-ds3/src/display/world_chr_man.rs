use hudhook::imgui::{TreeNodeFlags, Ui};

use darksouls3::sprj::*;
use shared::{Subclass, Superclass};

use super::DebugDisplay;

impl DebugDisplay for WorldChrMan {
    fn render_debug(&mut self, ui: &&mut Ui) {
        ui.text(format!(
            "World Area Chr Count: {}",
            self.world_area_chr_count
        ));

        let mut world_block_chrs = self.world_block_chrs_mut().collect::<Vec<_>>();
        if ui.collapsing_header(
            format!("World Block Chrs: {}", world_block_chrs.len()),
            TreeNodeFlags::empty(),
        ) {
            ui.indent();
            for (i, world_block_chr) in world_block_chrs.iter_mut().enumerate() {
                if ui.collapsing_header(format!("Block {}", i), TreeNodeFlags::empty()) {
                    ui.indent();
                    world_block_chr.render_debug(ui);
                    ui.unindent();
                }
            }
            ui.unindent();
        }

        ui.text(format!(
            "World Block Chr Count: {}",
            self.world_block_chr_count
        ));

        ui.text(format!(
            "Loaded? World Block Chr Count: {}",
            self.loaded_world_block_chr_count
        ));

        if ui.collapsing_header("Player ChrSet", TreeNodeFlags::empty()) {
            ui.indent();
            self.player_chr_set.render_debug(ui);
            ui.unindent();
        }

        if ui.collapsing_header("Ghost ChrSet", TreeNodeFlags::empty()) {
            ui.indent();
            self.ghost_chr_set.render_debug(ui);
            ui.unindent();
        }

        if ui.collapsing_header("Debug ChrSet", TreeNodeFlags::empty()) {
            ui.indent();
            self.debug_chr_set.render_debug(ui);
            ui.unindent();
        }

        match self.main_player.as_mut() {
            Some(p) => {
                if ui.collapsing_header("Main player", TreeNodeFlags::empty()) {
                    ui.indent();
                    unsafe { p.as_mut() }.render_debug(ui);
                    ui.unindent();
                }
            }
            None => ui.text("No Main player instance"),
        }
    }
}

impl<T> DebugDisplay for ChrSet<T>
where
    T: Subclass<ChrIns>,
{
    fn render_debug(&mut self, ui: &&mut Ui) {
        let mut characters = self.iter_mut().collect::<Vec<_>>();
        if ui.collapsing_header(
            format!("Characters: {}", characters.len()),
            TreeNodeFlags::empty(),
        ) {
            ui.indent();
            for chr_ins in characters.iter_mut() {
                if ui.collapsing_header(
                    format!("{} ##{:p}", chr_ins.id(), chr_ins),
                    TreeNodeFlags::empty(),
                ) {
                    let base = chr_ins.superclass_mut();
                    ui.indent();
                    if let Some(player_ins) = base.as_subclass_mut::<PlayerIns>() {
                        player_ins.render_debug(ui);
                    } else {
                        base.render_debug(ui);
                    }
                    ui.unindent();
                }
            }
            ui.unindent();
        }
    }
}
