use fromsoftware_shared::FromStatic;
use hudhook::imgui::{TreeNodeFlags, Ui};

pub(crate) mod chr;
pub(crate) mod event_flag;
pub(crate) mod field_area;
pub(crate) mod menu;
pub(crate) mod param;
pub(crate) mod world_block;
pub(crate) mod world_chr_man;

pub trait DebugDisplay {
    fn render_debug(&mut self, ui: &&mut Ui);
}

pub trait StatefulDebugDisplay {
    type State: Default;

    fn render_debug(&mut self, ui: &&mut Ui, state: &mut Self::State);
}

impl<T> StatefulDebugDisplay for T
where
    T: DebugDisplay,
{
    type State = ();

    fn render_debug(&mut self, ui: &&mut Ui, _state: &mut Self::State) {
        <Self as DebugDisplay>::render_debug(self, ui);
    }
}

pub struct StaticDebugger<T>
where
    T: StatefulDebugDisplay + FromStatic + 'static,
{
    state: T::State,
}

impl<T> StaticDebugger<T>
where
    T: StatefulDebugDisplay + FromStatic + 'static,
{
    pub fn new() -> Self {
        StaticDebugger {
            state: Default::default(),
        }
    }
}

impl<T> DebugDisplay for StaticDebugger<T>
where
    T: StatefulDebugDisplay + FromStatic + 'static,
{
    fn render_debug(&mut self, ui: &&mut Ui) {
        let singleton = unsafe { T::instance() };

        match singleton {
            Ok(instance) => {
                if ui.collapsing_header(
                    format!("{}: {:p}", T::name(), instance),
                    TreeNodeFlags::empty(),
                ) {
                    ui.indent();
                    instance.render_debug(ui, &mut self.state);
                    ui.unindent();
                    ui.separator();
                }
            }
            Err(err) => ui.text(format!("Couldn't load {}: {:?}", T::name(), err)),
        }
    }
}
