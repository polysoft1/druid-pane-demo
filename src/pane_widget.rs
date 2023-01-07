use druid::widget::{Widget, Flex, Label, Container};
use druid::{WidgetPod, WidgetExt, Color, Point};
use crate::PaneData;

pub struct PaneWidget {
    contents: WidgetPod<PaneData, Container<PaneData>>,
}

impl PaneWidget {
    pub fn new() -> Self {
        let placeholder_label = Label::new("Pane");
        PaneWidget { contents: WidgetPod::new(
                Flex::column()
                    .with_child(placeholder_label)
                    .expand()
                    .background(Color::GRAY)
            )
        }
    }
}


impl Widget<PaneData> for PaneWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut PaneData, env: &druid::Env) {
        self.contents.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &PaneData, env: &druid::Env) {
        self.contents.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, _old_data: &PaneData, data: &PaneData, env: &druid::Env) {
        self.contents.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &PaneData, env: &druid::Env) -> druid::Size {
        let layout_result = self.contents.layout(ctx, bc, data, env);
        self.contents.set_origin(ctx, Point::new(0.0, 0.0));
        layout_result
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &PaneData, env: &druid::Env) {
        self.contents.paint(ctx, data, env);
    }
}