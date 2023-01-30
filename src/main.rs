use druid::{AppLauncher, Color, Data, Lens, WindowDesc};
use druid::widget::EnvScope;
use druid::im;

mod pane_dock_widget;
mod pane_header_widget;
mod pane_widget;

const DEFAULT_PANE_HEIGHT: f64 = 480.0;
const DEFAULT_PANE_WIDTH: f64 = 300.0;
const PANE_SPACING: f64 = 8.0;
const HEADER_HEIGHT: f64 = 25.0;
const DOCK_BACKGROUND_COLOR: Color = Color::rgba8(0, 0, 0, 200);
const PANE_BACKGROUND_COLOR: Color = Color::rgb8(70, 70, 70);
const PANE_HEADER_COLOR: Color = Color::rgb8(40, 40, 40);

pub const DOCK_BACKGROUND_COLOR_KEY: druid::env::Key<druid::Color> = druid::env::Key::new("polysoft.druid-demo.dock-background-color");
pub const PANE_BACKGROUND_COLOR_KEY: druid::env::Key<druid::Color> = druid::env::Key::new("polysoft.druid-demo.pane-background-color");
pub const PANE_HEADER_COLOR_KEY: druid::env::Key<druid::Color> = druid::env::Key::new("polysoft.druid-demo.pane-header-color");


#[derive(Clone, Data, Lens)]
struct PaneData {
    id: i32,
}

#[derive(Clone, Data, Lens)]
struct AppState {
    show_dock: bool,
    always_on_top: bool,
    panes: im::Vector<PaneData>,
    next_pane_id: i32,
}

impl AppState {
    pub fn add_pane(&mut self) {
        self.panes.push_back(PaneData { id: self.next_pane_id });
        self.next_pane_id += 1;
    }
}

fn main() {
    let root = EnvScope::new(
        |env: &mut druid::env::Env, _data: &AppState| {
            env.set(DOCK_BACKGROUND_COLOR_KEY, DOCK_BACKGROUND_COLOR);
            env.set(PANE_BACKGROUND_COLOR_KEY, PANE_BACKGROUND_COLOR);
            env.set(PANE_HEADER_COLOR_KEY, PANE_HEADER_COLOR);
        },
        pane_dock_widget::PaneDockWidget::new()
    );
    let mut main_window = WindowDesc::new(root)
        .title("Pane Demo Main Win")
        .window_size((1024.0, DEFAULT_PANE_HEIGHT + HEADER_HEIGHT + 100.0))
        .transparent(true);
    if cfg!(target_os = "macos") {
        main_window = main_window.show_titlebar(false);
    }

    let state = AppState {
        show_dock: true,
        always_on_top: false,
        panes: im::vector![
            PaneData { id: 0 }
        ],
        next_pane_id: 1,
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("Failed to launch application");
}