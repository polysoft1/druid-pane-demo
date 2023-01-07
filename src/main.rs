use druid::{AppLauncher, Data, Lens, WindowDesc};
use druid::im;

mod pane_dock_widget;
mod pane_widget;

const DEFAULT_PANE_HEIGHT: f64 = 400.0;
const DEFAULT_PANE_WIDTH: f64 = 280.0;
const PANE_SPACING: f64 = 10.0;

#[derive(Clone, Data, Lens)]
struct PaneData {
    
}

#[derive(Clone, Data, Lens)]
struct AppState {
    show_dock: bool,
    panes: im::Vector<PaneData>
}

impl AppState {
    pub fn add_pane(&mut self) {
        self.panes.push_back(PaneData { });
    }
}

fn main() {
    let main_window = WindowDesc::new(pane_dock_widget::PaneDockWidget::new())
        .title("Pane Demo Main Win")
        .window_size((800.0, DEFAULT_PANE_HEIGHT + 90.0))
        .transparent(true);

    let state = AppState {
        show_dock: true,
        panes: im::vector![
            PaneData { }
        ]
    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("Failed to launch application");
}