use druid::widget::{Widget, Label, Padding, Container};
use druid::{WidgetPod, Point, WidgetExt, Color, BoxConstraints, Size};
use crate::PaneData;

pub struct PaneHeaderWidget {
    title: WidgetPod<PaneData, Padding<PaneData, Label<PaneData>>>,
    close_label: WidgetPod<PaneData, Container<PaneData>>,
}

impl PaneHeaderWidget {
    pub fn new() -> Self {
        let title = Label::new(|data: &PaneData, _: &_| format!("Pane {} header", data.id)).padding(3.0);
        let close_label = Label::new("✖").padding(3.0).expand_height().background(Color::rgba8(255, 255, 255, 20));
        Self { title: WidgetPod::new(title), close_label: WidgetPod::new(close_label) }
    }

    pub fn is_in_button(&self, point: Point) -> bool {
        self.close_label.layout_rect().contains(point)
    }
}


impl Widget<PaneData> for PaneHeaderWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut PaneData, env: &druid::Env) {
        self.title.event(ctx, event, data, env);
        self.close_label.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &PaneData, env: &druid::Env) {
        self.title.lifecycle(ctx, event, data, env);
        self.close_label.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, _old_data: &PaneData, data: &PaneData, env: &druid::Env) {
        self.title.update(ctx, data, env);
        self.close_label.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &PaneData, env: &druid::Env) -> druid::Size {
        let child_bc = BoxConstraints::new(Size::new(0.0, 0.0), bc.max());

        let _title_layout_result = self.title.layout(ctx, &child_bc, data, env);
        let close_label_layout_result = self.close_label.layout(ctx, &child_bc, data, env);

        self.title.set_origin(ctx, Point::new(0.0, 0.0));
        self.close_label.set_origin(ctx, Point::new(bc.max().width - close_label_layout_result.width, 0.0));
        bc.max()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &PaneData, env: &druid::Env) {
        self.title.paint(ctx, data, env);
        self.close_label.paint(ctx, data, env);
    }
}