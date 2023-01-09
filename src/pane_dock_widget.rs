use druid::widget::{Widget, Flex, Label, Button, Container, LineBreaking};
use druid::widget::prelude::*;
use druid::{WidgetPod, WidgetExt, Point, Color, Region};
use crate::{AppState, PaneData, pane_widget::PaneWidget};

pub struct PaneDockWidget {
    dock_items: WidgetPod<AppState, Flex<AppState>>,
    persistent_items: WidgetPod<AppState, Container<AppState>>,
    /// Order should match order in data, not x position order.
    panes: Vec<(PaneLocationData, WidgetPod<PaneData, PaneWidget>)>,
}
struct PaneLocationData {
    x_pos: f64, // Relative to the bound side. Default dist from right.
    width: f64,
    //is_minimized: bool,
    height: f64,
}

impl PaneDockWidget {
    pub fn new() -> Self {
        let info_label = Label::new("Move and resize the pane dock, then hide the dock.");

        let toggle_dock_button = Button::new("Toggle Dock")
            .on_click(|ctx, data: &mut bool, _: &Env| {
                *data = !*data;
                ctx.window().show_titlebar(*data);
                ctx.request_layout();
            })
            .lens(AppState::show_dock)
            .boxed();

        let add_pane_button = Button::new("Add Pane")
        .on_click(|ctx, data: &mut AppState, _: &Env| {
            data.add_pane();
            ctx.children_changed();
        });

        let mut dock_items = Flex::column()
            .with_child(info_label);
        if cfg!(target_os = "linux") {
            // Can't manually set this on Linux, but it is more often than not an option on the titlebar
            let mut always_on_top_msg: Label<AppState> = Label::new("Most desktop environments allow you to set this window always on top by right clicking on the titlebar and selecting \"Always on top\"");
            always_on_top_msg.set_line_break_mode(LineBreaking::WordWrap);
            dock_items.add_default_spacer();
            dock_items.add_child(always_on_top_msg);
        }
        if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
            let always_on_top_button = Button::new("Toggle Always On Top")
                .on_click(|ctx, data: &mut AppState, _: &Env| {
                    data.always_on_top = !data.always_on_top;
                    println!("Setting always on top to: {}", data.always_on_top);
                    ctx.window().set_always_on_top(data.always_on_top);
                });
            dock_items.add_child(always_on_top_button);
        }

        let persistent_items = Flex::column()
            .with_child(toggle_dock_button)
            .with_child(add_pane_button)
            .padding(3.0)
            .background(Color::rgba(255.0, 255.0, 255.0, 0.1));

        Self {
            dock_items: WidgetPod::new(dock_items),
            persistent_items: WidgetPod::new(persistent_items),
            panes: vec![],
        }
    }

    fn update_pane_widgets(&mut self, data: &AppState) -> bool {
        let panes_added = data.panes.len() - self.panes.len();
        if panes_added > 0 {
            // Added
            for _ in 0..panes_added {
                self.add_pane();
            }
        } else if panes_added < 0 {
            // Removed
            // TODO
        }
        panes_added != 0
    }

    fn get_next_position(&self) -> f64 {
        let mut next_pos = 0.0;
        for (pane_location_data, _pane_widget) in &self.panes {
            let far_edge = pane_location_data.x_pos + pane_location_data.width;
            if far_edge > next_pos {
                next_pos = far_edge;
            }
        }
        next_pos
    }

    fn add_pane(&mut self) {
        let next_pos = self.get_next_position();
        let new_loc_data = PaneLocationData {
            height: crate::DEFAULT_PANE_HEIGHT,
            width: crate::DEFAULT_PANE_WIDTH,
            x_pos: next_pos + crate::PANE_SPACING,
        };
        let new_widget = WidgetPod::new(PaneWidget::new());
        self.panes.push((new_loc_data, new_widget));
    }
}

impl Widget<AppState> for PaneDockWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut AppState, env: &druid::Env) {
        self.dock_items.event(ctx, event, data, env);
        self.persistent_items.event(ctx, event, data, env);

        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter_mut();
        for ((_pane_location_data,pane_widget), pane_data) in panes_iter.zip(data_iter) {
            pane_widget.event(ctx, event, pane_data, env);
        };
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &AppState, env: &druid::Env) {
        self.dock_items.lifecycle(ctx, event, data, env);
        self.persistent_items.lifecycle(ctx, event, data, env);

        if let LifeCycle::WidgetAdded = event {
            // When added, make sure all existing panes are accounted for
            if self.update_pane_widgets(data) {
                ctx.children_changed();
                ctx.request_layout();
            }
        }

        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter();
        for ((_pane_location_data,pane_widget), pane_data) in panes_iter.zip(data_iter) {
            pane_widget.lifecycle(ctx, event, pane_data, env);
        };
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &AppState, data: &AppState, env: &druid::Env) {
        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter();
        for ((_pane_location_data,pane_widget), pane_data) in panes_iter.zip(data_iter) {
            pane_widget.update(ctx, pane_data, env);
        };

        if self.update_pane_widgets(data) || old_data.show_dock != data.show_dock {
            ctx.children_changed();
            ctx.request_layout();
        }
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &AppState, env: &druid::Env) -> druid::Size {
        let mut iteractable_area = Region::EMPTY;
        let inner_item_bc = BoxConstraints::new(Size::new(0.0, 0.0), bc.max());
        
        // Position to right
        let persistent_items_size = self.persistent_items.layout(ctx, &inner_item_bc, data, env);
        self.persistent_items.set_origin(ctx, Point::new(bc.max().width - persistent_items_size.width, 0.0));
        iteractable_area.add_rect(self.persistent_items.layout_rect());

        if data.show_dock {
            let dock_item_bc = BoxConstraints::new(inner_item_bc.min(),
                Size::new(inner_item_bc.max().width - persistent_items_size.width, inner_item_bc.max().height));
            let _dock_items_layout = self.dock_items.layout(ctx, &dock_item_bc, data, env);
            self.dock_items.set_origin(ctx, Point::new(0.0, 0.0));
            iteractable_area.add_rect(bc.max().to_rect().inflate(40.0, 80.0));
        }

        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter();
        for ((pane_location_data,pane_widget), pane_data) in panes_iter.zip(data_iter) {
            let pane_bc = BoxConstraints::new(
                Size::new(pane_location_data.width, pane_location_data.height), 
                Size::new(pane_location_data.width, pane_location_data.height)
            );
            let pane_size = pane_widget.layout(ctx, &pane_bc, pane_data, env);
            // align to bottom
            pane_widget.set_origin(ctx, Point::new(
                bc.max().width - pane_location_data.x_pos - pane_location_data.width,
                bc.max().height - pane_size.height
            ));
            // Make it interactable
            iteractable_area.add_rect(pane_widget.layout_rect());
        };
        if data.show_dock {
            ctx.window().set_interactable_area(None);
        } else {
            ctx.window().set_interactable_area(Some(iteractable_area));
        }

        bc.max()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppState, env: &druid::Env) {
        if data.show_dock {
            // Paint background
            let paint_area = ctx.region().bounding_box();
            ctx.fill(paint_area, &Color::rgba(0.2, 0.2, 0.2, 0.7));
            // Paint dock items
            self.dock_items.paint(ctx, data, env);
        }

        self.persistent_items.paint(ctx, data, env);

        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter();
        for ((_pane_location_data,pane_widget), pane_data) in panes_iter.zip(data_iter) {
            pane_widget.paint(ctx, pane_data, env);
        };
    }
}