use ::shared::FromStatic;
use from_singleton::FromSingleton;
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
    name: String,
    state: T::State,
}

impl<T> StaticDebugger<T>
where
    T: StatefulDebugDisplay + FromStatic + 'static,
{
    pub fn new(name: impl AsRef<str>) -> Self {
        StaticDebugger {
            name: name.as_ref().to_string(),
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
                    format!("{}: {:p}", self.name, instance),
                    TreeNodeFlags::empty(),
                ) {
                    ui.indent();
                    instance.render_debug(ui, &mut self.state);
                    ui.unindent();
                    ui.separator();
                }
            }
            Err(err) => ui.text(format!("Couldn't load {}: {:?}", self.name, err)),
        }
    }
}

pub struct SingletonDebugger<T>(StaticDebugger<T>)
where
    T: StatefulDebugDisplay + FromSingleton + 'static;

impl<T> SingletonDebugger<T>
where
    T: StatefulDebugDisplay + FromSingleton + 'static,
{
    pub fn new() -> Self {
        SingletonDebugger(StaticDebugger::new(T::name()))
    }
}

impl<T> DebugDisplay for SingletonDebugger<T>
where
    T: StatefulDebugDisplay + FromSingleton + 'static,
{
    fn render_debug(&mut self, ui: &&mut Ui) {
        DebugDisplay::render_debug(&mut self.0, ui);
    }
}
