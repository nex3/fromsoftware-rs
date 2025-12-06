use std::time::Duration;

use darksouls3::cs::*;
use darksouls3::sprj::*;
use darksouls3::util::{input::*, system::wait_for_system_init};
use hudhook::hooks::dx11::ImguiDx11Hooks;
use hudhook::windows::Win32::Foundation::HINSTANCE;
use hudhook::{eject, imgui::*, Hudhook, ImguiRenderLoop};
use shared::Program;
use tracing_panic::panic_hook;

mod display;

use display::{DebugDisplay, SingletonDebugger, StaticDebugger};

/// # Safety
/// This is exposed this way such that libraryloader can call it. Do not call this yourself.
#[no_mangle]
pub unsafe extern "C" fn DllMain(hmodule: HINSTANCE, reason: u32) -> bool {
    if reason == 1 {
        std::panic::set_hook(Box::new(panic_hook));

        let appender = tracing_appender::rolling::never("./", "chains-debug.log");
        tracing_subscriber::fmt().with_writer(appender).init();

        let blocker =
            unsafe { InputBlocker::get_instance() }.expect("Failed to initialize input blocker");

        std::thread::spawn(move || {
            wait_for_system_init(&Program::current(), Duration::MAX)
                .expect("Timeout waiting for system init");

            if let Err(e) = Hudhook::builder()
                .with::<ImguiDx11Hooks>(DarkSouls3DebugGui::new(blocker))
                .with_hmodule(hmodule)
                .build()
                .apply()
            {
                tracing::error!("Couldn't apply hooks: {e:?}");
                eject();
            }
        });
    }

    true
}

struct DarkSouls3DebugGui {
    input_blocker: &'static InputBlocker,
    size: [f32; 2],
    scale: f32,
    world: SingletonDebugger<WorldChrMan>,
    field_area: StaticDebugger<FieldArea>,
    events: SingletonDebugger<SprjEventFlagMan>,
    item_get_menu_man: StaticDebugger<ItemGetMenuMan>,
    params: SingletonDebugger<CSRegulationManager>,
}

impl DarkSouls3DebugGui {
    fn new(input_blocker: &'static InputBlocker) -> Self {
        Self {
            input_blocker,
            size: [600., 400.],
            scale: 1.8,
            world: SingletonDebugger::new(),
            field_area: StaticDebugger::new("FieldArea"),
            events: SingletonDebugger::new(),
            item_get_menu_man: StaticDebugger::new("ItemGetMenuMan"),
            params: SingletonDebugger::new(),
        }
    }
}

impl ImguiRenderLoop for DarkSouls3DebugGui {
    fn initialize(&mut self, _ctx: &mut Context, _render_context: &mut dyn hudhook::RenderContext) {
        // TODO: Look for CSWindowImp and scale everything based on that like ER
        // does.
    }

    fn render(&mut self, ui: &mut Ui) {
        let io = ui.io();
        let mut flag = InputFlags::empty();
        if io.want_capture_mouse {
            flag = flag | InputFlags::Mouse;
        }
        if io.want_capture_keyboard {
            flag = flag | InputFlags::Keyboard;
        }
        if io.want_capture_mouse && io.want_capture_keyboard {
            // Only block pad input if both the mouse and keyboard are blocked
            // (for example if a modal dialog is up).
            flag = flag | InputFlags::GamePad;
        }
        self.input_blocker.block_only(flag);

        ui.window("Dark Souls III Rust Bindings Debug")
            .position([30., 30.], Condition::FirstUseEver)
            .size(self.size, Condition::FirstUseEver)
            .build(|| {
                ui.set_window_font_scale(self.scale);
                let tabs = ui.tab_bar("main-tabs").unwrap();
                if let Some(item) = ui.tab_item("World") {
                    self.world.render_debug(&ui);
                    self.events.render_debug(&ui);
                    self.field_area.render_debug(&ui);
                    item.end();
                }

                if let Some(item) = ui.tab_item("Menu") {
                    self.item_get_menu_man.render_debug(&ui);
                    item.end();
                }

                if let Some(item) = ui.tab_item("Resource") {
                    self.params.render_debug(&ui);
                    item.end();
                }

                if let Some(item) = ui.tab_item("Eject") {
                    if ui.button("Eject") {
                        eject();
                    }
                    item.end();
                }
                tabs.end();
            });
    }
}
