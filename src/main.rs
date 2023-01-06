use druid::{AppLauncher, Color, Data, Lens, Rect, Region, Size, WindowConfig, WindowDesc, WindowLevel, WidgetExt};
use druid::widget::{Button, Flex, Label, Widget, CrossAxisAlignment};

const PANE_HEIGHT: f64 = 400.0;

#[derive(Clone, Data, Lens)]
struct AppState {
    show_dock: bool,

}


fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title("Pane Demo Main Win")
        .window_size((800.0, PANE_HEIGHT + 90.0))
        .transparent(true);

    let state = AppState {
        show_dock: true,

    };

    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<AppState> {
    let label = Label::new("Main Pane Window");
    let add_pane_button = Button::new("Make sub window")
        .on_click(|ctx, data: &mut AppState, env| {
            ctx.new_sub_window(
                WindowConfig::default()
                    .show_titlebar(true)
                    .window_size(Size::new(250.0, PANE_HEIGHT))
                    .set_level(WindowLevel::Tooltip(ctx.window().clone())),
                build_pane_widget(),
                data.clone(),
                env.clone(),
            );
        }
    );
    let show_titlebar = Button::new("Show Titlebar")
        .on_click(|ctx, _data: &mut AppState, _env| {
            ctx.window().show_titlebar(true);
        }
    );
    let hide_titlebar = Button::new("Hide Titlebar")
        .on_click(|ctx, _data: &mut AppState, _env| {
            ctx.window().show_titlebar(false);
        }
    );
    let hide_dock = Button::new("Set Region")
        .on_click(|ctx, _data: &mut AppState, _env| {
            let mut region = Region::EMPTY;
            region.add_rect(Rect::new(0.0, 0.0, 800.0, 50.0));
            ctx.window().set_interactable_area(&region);
        }
    );
    Flex::row()
        .with_child(label)
        .with_default_spacer()
        .with_child(add_pane_button)
        .with_default_spacer()
        .with_child(show_titlebar)
        .with_default_spacer()
        .with_child(hide_titlebar)
        .with_default_spacer()
        .with_child(hide_dock)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .expand()
        .background(Color::rgba(5.0, 5.0, 5.0, 0.25))
}

fn build_pane_widget() -> impl Widget<AppState> {
    let label = Label::new("This is a pane");
    let position_within_dock = Button::new("TODO")
        .on_click(|_ctx, _data: &mut AppState, _env| {
            //ctx.window().set_position_within_parent(Point::new(0.0, 0.0));
        }
    );
    Flex::column()
        .with_child(label)
        .with_child(position_within_dock)
}
