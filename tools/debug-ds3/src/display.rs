use ::shared::FromStatic;
use from_singleton::FromSingleton;
use hudhook::imgui::{TreeNodeFlags, Ui};

pub(crate) mod chr;
pub(crate) mod param;
pub(crate) mod world_block;
pub(crate) mod world_chr_man;

pub trait DebugDisplay {
    fn render_debug(&mut self, ui: &&mut Ui);
}

pub fn render_debug_singleton<T: DebugDisplay + FromSingleton + 'static>(ui: &&mut Ui) {
    let singleton = unsafe { T::instance() };

    match singleton {
        Ok(instance) => {
            if ui.collapsing_header(T::name(), TreeNodeFlags::empty()) {
                ui.indent();
                let pointer = instance as *const T;
                let mut pointer_string = format!("{pointer:#x?}");
                let label = format!("{} instance", T::name());
                ui.input_text(label.as_str(), &mut pointer_string)
                    .read_only(true)
                    .build();

                instance.render_debug(ui);
                ui.unindent();
                ui.separator();
            }
        }
        Err(err) => ui.text(format!("Couldn't load {}: {:?}", T::name(), err)),
    }
}
