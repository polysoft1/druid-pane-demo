use druid::widget::{Widget, Flex, Label, Button, Container, LineBreaking};
use druid::widget::prelude::*;
use druid::{WidgetPod, WidgetExt, Point, Color, Region, Rect};
use crate::HEADER_HEIGHT;
use crate::{AppState, PaneData, pane_widget::PaneWidget, pane_header_widget::PaneHeaderWidget};

pub struct PaneDockWidget {
    dock_items: WidgetPod<AppState, Flex<AppState>>,
    persistent_items: WidgetPod<AppState, Container<AppState>>,
    /// Order should match order in data, not x position order.
    // TODO: Maybe split this up. Something related to removing one of these is causing a bug.
    panes: Vec<(PaneLocationData, WidgetPod<PaneData, PaneWidget>, WidgetPod<PaneData, PaneHeaderWidget>)>,
    // For dragging the dock and panes
    init_pos: Option<Point>,
    dragging_pane: Option<usize>,
    last_movements: Vec<f64>,
    last_movements_index: usize,
    total_drag_dist: f64,
}
struct PaneLocationData {
    // Positions are relative to the bound side. Default dist from right.
    target_x_pos: f64, // For keeping track of where it needs to go to fit into the dock
    actual_x_pos: f64, // For keeping track of actual location for animating it into place
    width: f64,
    //is_minimized: bool,
    height: f64,
}

impl PaneDockWidget {
    pub fn new() -> Self {
        let info_label = Label::new("Move and resize the pane dock, then hide the dock.");

        let toggle_dock_button = Button::new("Toggle Dock")
            .on_click(|ctx, is_shown: &mut bool, _: &Env| {
                *is_shown = !*is_shown;
                ctx.window().show_titlebar(*is_shown);
                // Resize to refresh background.
                let old_size = ctx.window().get_size();
                let new_size = if *is_shown {
                    // Shrink in height by 1 px
                    Size::new(old_size.width, old_size.height - 1.0)
                } else {
                    // Grow in height by 1 px
                    Size::new(old_size.width, old_size.height + 1.0)
                };
                ctx.window().set_size(new_size);
                ctx.request_layout();
            })
            .lens(AppState::show_dock)
            .boxed();

        let add_pane_button = Button::new("Add Pane")
        .on_click(|ctx, data: &mut AppState, _: &Env| {
            data.add_pane();
            ctx.children_changed();
        });

        let mut dock_items = Flex::row()
            .with_child(info_label);
        if cfg!(target_os = "linux") {
            // Can't manually set this on Linux, but it is more often than not an option on the titlebar
            let mut always_on_top_msg: Label<AppState> = Label::new("If the always on top button does nothing, most desktop environments allow you to set this window always on top by right clicking on the titlebar and selecting \"Always on top\"");
            always_on_top_msg.set_line_break_mode(LineBreaking::WordWrap);
            dock_items.add_default_spacer();
            dock_items.add_child(always_on_top_msg);
        }
        let always_on_top_button = Button::new("Toggle Always On Top")
            .on_click(|ctx, data: &mut AppState, _: &Env| {
                data.always_on_top = !data.always_on_top;
                println!("Setting always on top to: {}", data.always_on_top);
                ctx.window().set_always_on_top(data.always_on_top);
            });
        let close_window_button = Button::new("Close Window")
            .on_click(|ctx, _: &mut AppState, _| {
                ctx.window().close();
            });
        dock_items.add_child(close_window_button);
        dock_items.add_child(always_on_top_button);

        let persistent_items = Flex::column()
            .with_child(toggle_dock_button)
            .with_child(add_pane_button)
            .padding(3.0)
            .background(Color::rgba(255.0, 255.0, 255.0, 0.1));

        Self {
            dock_items: WidgetPod::new(dock_items),
            persistent_items: WidgetPod::new(persistent_items),
            panes: vec![],
            init_pos: None,
            dragging_pane: None,
            last_movements: vec![0.0; 15 as usize], // four zero values
            last_movements_index: 0,
            total_drag_dist: 0.0,
        }
    }

    /// Returns true if panes changed
    fn update_pane_widgets(&mut self, data: &AppState, old_data: Option<&AppState>) -> bool {
        // The situations are:
        //  1. - 1 or more are added, ideally at the end, but it could be anywhere.
        //  2. - 1 or more are removed, anywhere.
        //  3. - 1 or more are added, and 1 or more are removed at the same time
        // So we need to compare, and match all unchanged indexes,
        // and for the changed indexes, either add panes, remove panes, or replace panes.

        // Situation 1:
        // Original: 1 2 3 4 5
        //  Changed: 1 2 6 4 5
        // In this situation, the 3 was replaced with 6, so the widget for 3 is removed, and added for 6.
        // Situation 2:
        // Original: 1 2 3
        //  Changed: 1 2 4 3
        // In this situation, a new widget should get created for 4, instead of creating one for 3 and shifting them.
        // Situation 3:
        // Original: 1 2 3
        //  Changed: 1 3
        // In this situation, the widget for 2 should be removed, as opposed to the one for 3.

        // Plan: Iterate through both old and new data until the end is reached, individually comparing
        // the data points for all widgets keeping track of necessary additions and deletions.
        // This may need to be an n^2 algorithm, checking the data and then iterating iterating through
        // the other list to find it.

        // Position new panes where they need to go. The other panes will need to shift themselves to adjust
        // Do not reposition panes next to removed panes. They will need to shift themselves.
        match old_data {
            Some(old_data) => {
                let mut changed = false;

                for i in (0..old_data.panes.len()).rev() { // Back to front to prevent changing the indexes
                    // Assuming it was properly managed before, the i index will match between the old data and the widgets
                    // Check to see if the data at index i is still there. If not, remove the data at that index
                    let mut new_data_itr = data.panes.iter();
                    let old_pane_data = &old_data.panes[i];
                    let old_in_new_position = new_data_itr.position(|x| x.id == old_pane_data.id);
                    if old_in_new_position.is_none() {
                        println!("No pane with ID {} found in new data. Removing pane at index {}", old_pane_data.id, i);
                        self.panes.remove(i);
                        changed = true;
                    } else {
                        println!("Pane with ID {} is still in the new data.", old_pane_data.id);
                    }
                }
                for old_pane_data in &old_data.panes {
                    println!("Old data ID: {}", old_pane_data.id);
                }
                // For the second pass, it will be iterating, finding new ones, and adding the widgets
                //let mut expected_old_index = 0;
                for i in 0..data.panes.len() { // Position now relative to the desired state
                    // While iterating, keep track of the position
                    // If the item doesn't match an item for the old data, add new data, but do
                    // not increment the position in the old data iterator.

                    let mut old_data_itr = old_data.panes.iter();
                    println!("Checking pane at index {i}");
                    let new_data_item = &data.panes[i];

                    // First check to see if the new data it's in the old data.
                    // If it is not, it should be added right here as a new widget.
                    let new_in_old_data_position = old_data_itr.position(|x| {
                        println!("Checking old pane with ID {}. Searching for {}", x.id, new_data_item.id);
                        x.id == new_data_item.id
                    });

                    if let Some(_new_in_old_data_position) = new_in_old_data_position {
                        // If, for example, the first index is removed, the expected index in the old data will be 1
                        println!("Found old pane for ID {}. Moving on.", new_data_item.id);
                        // Existing item, so see if it was reordered.
                        // If the item at expected_old_index does not equal the new_data_item, it was reordered.

                        /*if new_in_old_data_position != expected_old_index {
                            eprintln!("Unexpected position. Expected {}, got {}. Exiting early", new_in_old_data_position, expected_old_index);
                        }*/
                        // Lastly, add to the old index because this is an existing item.
                        //expected_old_index += 1;
                    } else {
                        // New item, so add a widget for it
                        println!("Could not find position of pane with ID {}. Adding new pane.", new_data_item.id);
                        self.add_pane(i);
                        changed = true;
                        // Do not increment expected_old_index since it is a new addition
                    }
                }
                if changed {
                    self.refresh_all_target_positions();
                }
                println!("Changed: {changed}");
                println!("Data size: {}, panes size: {}", data.panes.len(), self.panes.len());
                return changed
            },
            None => {
                // This is probably the first run
                let panes_added = data.panes.len() - self.panes.len();
                for _ in 0..panes_added {
                    println!("Adding initial pane.");
                    self.add_pane(0);
                }
                return panes_added != 0
            }
        }
    }

    fn get_next_position(&self) -> f64 {
        let mut next_pos = 0.0;
        for (pane_location_data, _pane_widget, _pane_header_widget) in &self.panes {
            let far_edge = pane_location_data.target_x_pos + pane_location_data.width;
            if far_edge > next_pos {
                next_pos = far_edge;
            }
        }
        next_pos
    }

    // For finding where to place a pane when moving it
    // It does this based on target locations, not actual locations
    // The exclude param is to not check the pane you're repositioning
    // It does it based on the center of the panes
    fn get_closest_pane_to_right(&self, x_pos: f64, exclude: usize) -> Option<usize> {
        if self.panes.len() <= 1 {
            None
        } else {
            let mut closest_position = 0.0;
            let mut closest_position_pane_index: Option<usize> = None;
            for (i, (position_data, _pane_widget, _header_widget)) in self.panes.iter().enumerate() {
                let pane_location_center = position_data.target_x_pos + position_data.width / 2.0;
                if i != exclude && pane_location_center > closest_position && pane_location_center < x_pos {
                    closest_position = pane_location_center;
                    closest_position_pane_index = Some(i);
                }
            }
            closest_position_pane_index
        }
    }

    fn get_new_target_x(&self, current_position: f64, exclude: usize) -> f64 {
        if let Some(pane_to_align_to) = self.get_closest_pane_to_right(current_position, exclude) {
            let align_pane_position_data = &self.panes[pane_to_align_to].0;
            align_pane_position_data.target_x_pos + align_pane_position_data.width + crate::PANE_SPACING
        } else {
            crate::PANE_SPACING
        }
    }

    // Makes sure that all panes that are at or left of moved_index are correctly to the left
    fn shift_left_of(&mut self, moved_index: usize) {
        // First find the left position
        let moved_pane_location_data = &self.panes[moved_index].0;
        let moved_pane_right_x = moved_pane_location_data.target_x_pos;
        let moved_pane_left_x = moved_pane_right_x + moved_pane_location_data.width;
        let mut shift_amount = 0.0;
        for i in 0..self.panes.len() {
            if i != moved_index {
                let pane_location = self.panes[i].0.target_x_pos;
                if pane_location >= moved_pane_right_x && pane_location < moved_pane_left_x {
                    let needed_shift = moved_pane_left_x - pane_location;
                    if needed_shift >= shift_amount {
                        shift_amount = needed_shift;
                    }
                }
            }
        }
        // Now shift everything at or left of the moved pane
        if shift_amount > 0.0 {
            for i in 0..self.panes.len() {
                if i != moved_index {
                    let pane_location = self.panes[i].0.target_x_pos;
                    if pane_location >= moved_pane_right_x {
                        self.panes[i].0.target_x_pos += shift_amount;
                    }
                }
            }
        }
    }

    // This refreshes them based on the centers of the panes
    fn refresh_all_target_positions(&mut self) {
        if self.panes.len() == 0 {
            return;
        }
        // Make sure the rightmost one is against the right. If not, shift them all
        let correct_right = crate::PANE_SPACING;
        let mut actual_rightmost = self.panes[0].0.target_x_pos;
        for i in 1..self.panes.len() {
            let existing_pane_pos = self.panes[i].0.target_x_pos;
            if existing_pane_pos < actual_rightmost {
                actual_rightmost = existing_pane_pos;
            }
        }
        let right_align_diff = actual_rightmost - correct_right;
        if right_align_diff > 0.0 {
            for i in 0..self.panes.len() {
                let pane_position_data = &mut self.panes[i].0;
                pane_position_data.target_x_pos -= right_align_diff;
            }
        }

        // Now align them all
        for i in 0..self.panes.len() {
            let position_data = &self.panes[i].0;
            let center_x = position_data.target_x_pos + position_data.width / 2.0;
            let new_target_x = self.get_new_target_x(center_x, i);
            let is_moved = new_target_x != position_data.target_x_pos;
            (&mut self.panes[i].0).target_x_pos = new_target_x;
            //if self.dragging_pane.is_none() || self.dragging_pane.unwrap() != i { // Don't snap the currently dragged one
            //    (&mut self.panes[i].0).actual_x_pos = new_target_x;
            //}
            // Now move all panes that aren't at index i that have an equal or greater position to left, if needed.
            // First check if there is an overlapping pane
            let mut has_overlapping = false;
            for j in 0..self.panes.len() {
                if j != i {
                    //println!("j : {}, j-x: {}, i-x: {}", j, self.panes[j].0.target_x_pos, new_target_x);
                }
                if j != i && self.panes[j].0.target_x_pos == new_target_x {
                    has_overlapping = true;
                    break;
                }
            }
            if has_overlapping {
                //println!("HAS OVERLAPPING");
                // Shift all panes to the left width display units to left
                let move_amount = self.panes[i].0.width + crate::PANE_SPACING;
                for j in 0..self.panes.len() {
                    self.panes[i].0.target_x_pos;
                    if j != i && self.panes[j].0.target_x_pos >= new_target_x {
                        self.panes[j].0.target_x_pos += move_amount;
                    }
                }
            }
        }
    }

    fn add_pane(&mut self, position: usize) {
        let next_pos = self.get_next_position() + crate::PANE_SPACING;
        let new_loc_data = PaneLocationData {
            height: crate::DEFAULT_PANE_HEIGHT,
            width: crate::DEFAULT_PANE_WIDTH,
            target_x_pos: next_pos,
            actual_x_pos: next_pos,
        };
        let new_content_widget = WidgetPod::new(PaneWidget::new());
        let new_header_widget = WidgetPod::new(PaneHeaderWidget::new());
        let new_item = (new_loc_data, new_content_widget, new_header_widget);
        self.panes.insert(position, new_item);
    }

    fn handle_dragging(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut AppState) {
        match event {
            Event::MouseDown(me) if me.buttons.has_left() => {
                self.total_drag_dist = 0.0;
                if self.get_input_area().contains(me.window_pos) {
                    // Now check to see if it's within a pane header
                    if let Some(widget_index) = self.find_pane(me.window_pos) {
                        let (_location_data, _contents, header) = &self.panes[widget_index];
                        if header.layout_rect().contains(me.window_pos) {
                            ctx.set_active(true);
                            self.init_pos = Some(me.window_pos);
                            self.dragging_pane = Some(widget_index);
                            // Start animating
                            ctx.request_anim_frame();
                        }
                    }
                } else {
                    ctx.set_active(true);
                    self.init_pos = Some(me.window_pos);
                    self.dragging_pane = None;
                }
            }
            Event::MouseMove(me) if ctx.is_active() && me.buttons.has_left() => {
                if let Some(init_pos) = self.init_pos {
                    let within_window_change = me.window_pos.to_vec2() - init_pos.to_vec2();
                    // Could either be dragging a pane or the entire window
                    if let Some(dragging_pane) = self.dragging_pane {
                        // Dragging pane
                        let location_data = &mut self.panes[dragging_pane].0;
                        // Move the pane's actual position based on the change
                        location_data.actual_x_pos -= within_window_change.x;
                        self.total_drag_dist += within_window_change.x.abs();
                        // Know if moving right, since the threshold changes between moving left and right
                        // Add the newest movement, and increment the circular index
                        if within_window_change.x.abs() > 1.0 {
                            self.last_movements[self.last_movements_index] = within_window_change.x;
                            self.last_movements_index += 1;
                            self.last_movements_index %= self.last_movements.len();
                        }

                        let mut total_recent_movement = 0.0;
                        for i in self.last_movements.iter() {
                            total_recent_movement += i;
                        }
                        let is_moving_right = total_recent_movement > 0.0;
                        // See if there is a new target location for this pane
                        // If so, adjust the target location.
                        let current_x = if is_moving_right {
                            location_data.actual_x_pos - location_data.width * 0.25
                        } else {
                            location_data.actual_x_pos + location_data.width * 1.25
                        };
                        let new_target_x = self.get_new_target_x(current_x, dragging_pane);
                        let location_data = &mut self.panes[dragging_pane].0;
                        location_data.target_x_pos = new_target_x;
                        // Make sure everything left is actually left of it
                        self.shift_left_of(dragging_pane);
                        // Refresh all
                        self.refresh_all_target_positions();

                        self.init_pos = Some(me.window_pos); // Move it to prevent compounding changes
                        ctx.request_layout();
                    } else {
                        // Dragging window
                        let old_pos = ctx.window().get_position();
                        let new_pos = old_pos + within_window_change;

                        ctx.window().set_position(new_pos)
                    }
                }
            }
            Event::MouseUp(me) if ctx.is_active() => {
                // Check to see if it was in the close button
                if let Some(dragging_pane) = self.dragging_pane {

                    // Check to see if it was on the close button
                    let (_location_data, _contents, header) = &mut self.panes[dragging_pane];
                    let header_position = header.layout_rect();
                    let relative_position = Point::new(me.window_pos.x - header_position.x0, me.window_pos.y - header_position.y0);

                    if self.total_drag_dist < 1.0 && header.widget().is_in_button(relative_position) {
                        data.panes.remove(dragging_pane);
                        self.refresh_all_target_positions();
                        ctx.request_update();
                        ctx.request_layout();
                    } else {
                        // Finalize the position
                        self.dragging_pane = None;
                        self.refresh_all_target_positions();
                        ctx.request_layout();
                    }
                }
                // Reset everything
                self.init_pos = None;
                self.dragging_pane = None;
                self.total_drag_dist = 0.0;
                ctx.set_active(false)
            }
            Event::AnimFrame(time_since_last_frame) => {
                // TODO: use time_since_last_frame to adjust to different frame rates
                // Check if any panes need animating. If they do, request another AnimFrame
                let expected_time = 16000000.0;
                let time_relative_to_reference = *time_since_last_frame as f64 / expected_time;
                let anim_speed_correction = time_relative_to_reference.max(0.25).min(1.25);
                let mut animation_needed = false;
                let mut layout_needed = false;
                let min_animation_speed = 20.0 * anim_speed_correction;
                for i in 0..self.panes.len() {
                    if !self.dragging_pane.is_none() && self.dragging_pane.unwrap() == i {
                        // Skip dragged pane until it's no longer being dragged
                        continue;
                    }
                    let existing_pane_location_data = &mut self.panes[i].0;
                    let location_diff = existing_pane_location_data.target_x_pos - existing_pane_location_data.actual_x_pos;
                    if location_diff.abs() > 0.0 {
                        let move_amount = if location_diff.abs() < min_animation_speed {
                            location_diff
                        } else {
                            let mut move_amount_magnitude = min_animation_speed;
                            
                            move_amount_magnitude += location_diff.abs() * 0.15 * anim_speed_correction;
                            if location_diff.is_sign_negative() {
                                move_amount_magnitude *= -1.0;
                            }
                            move_amount_magnitude
                        };
                        existing_pane_location_data.actual_x_pos += move_amount;
                        if !layout_needed {
                            layout_needed = true;
                        }

                        // Now determine if more change is required
                        if !animation_needed {
                            let new_location_diff = existing_pane_location_data.target_x_pos - existing_pane_location_data.actual_x_pos;
                            if new_location_diff.abs() > 0.0 {
                                animation_needed = true;
                            }
                        }
                    }
                }
                // No changes may be needed except for the currently dragged pane,
                // so keep requesting frames until dragging is complete.
                if animation_needed || self.dragging_pane.is_some() {
                    ctx.request_anim_frame();
                }
                if layout_needed {
                    ctx.request_layout();
                }
            }
            _ => (),
        }
    }

    fn get_input_area(&self) -> Region {
        let mut interactable_area = Region::EMPTY;
        interactable_area.add_rect(self.persistent_items.layout_rect());
        let panes_itr = self.panes.iter();
        for (_, pane_widget, header_widget) in panes_itr {
            interactable_area.add_rect(pane_widget.layout_rect());
            interactable_area.add_rect(header_widget.layout_rect());
        };
        interactable_area
    }

    fn find_pane(&self, point: Point) -> Option<usize> {
        for (i, (_, pane_widget, header_widget)) in self.panes.iter().enumerate() {
            if pane_widget.layout_rect().contains(point) || header_widget.layout_rect().contains(point) {
                return Some(i);
            }
        };
        None
    }
}

impl Widget<AppState> for PaneDockWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut AppState, env: &druid::Env) {
        self.dock_items.event(ctx, event, data, env);
        self.persistent_items.event(ctx, event, data, env);

        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter_mut();
        for ((_pane_location_data,pane_widget, header_widget), pane_data) in panes_iter.zip(data_iter) {
            pane_widget.event(ctx, event, pane_data, env);
            header_widget.event(ctx, event, pane_data, env);
        };

        self.handle_dragging(ctx, event, data);
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &AppState, env: &druid::Env) {
        self.dock_items.lifecycle(ctx, event, data, env);
        self.persistent_items.lifecycle(ctx, event, data, env);

        if let LifeCycle::WidgetAdded = event {
            // When added, make sure all existing panes are accounted for
            println!("Widget Added Event");
            if self.update_pane_widgets(data, None) {
                
                ctx.children_changed();
                ctx.request_layout();
                println!("Children changed and layout requested.");
            }
        }

        //println!("Pane data length: {}, Pane widget length: {}", data.panes.len(), self.panes.len());
        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter();
        for ((_pane_location_data,pane_widget, header_widget), pane_data) in panes_iter.zip(data_iter) {
            header_widget.lifecycle(ctx, event, pane_data, env);
            pane_widget.lifecycle(ctx, event, pane_data, env);
        };
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &AppState, data: &AppState, env: &druid::Env) {
        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter();
        for ((_pane_location_data,pane_widget, header_widget), pane_data) in panes_iter.zip(data_iter) {
            header_widget.update(ctx, pane_data, env);
            pane_widget.update(ctx, pane_data, env);
        };

        if self.update_pane_widgets(data, Some(old_data)) || old_data.show_dock != data.show_dock {
            ctx.children_changed();
            ctx.request_layout();
        }
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &AppState, env: &druid::Env) -> druid::Size {
        //println!("Layout");
        let inner_item_bc = BoxConstraints::new(Size::new(0.0, 0.0), bc.max());
        
        // Position to right
        let persistent_items_size = self.persistent_items.layout(ctx, &inner_item_bc, data, env);
        self.persistent_items.set_origin(ctx, Point::new(bc.max().width - persistent_items_size.width, 0.0));

        if data.show_dock {
            let dock_item_bc = BoxConstraints::new(inner_item_bc.min(),
                Size::new(inner_item_bc.max().width - persistent_items_size.width, inner_item_bc.max().height));
            let _dock_items_layout = self.dock_items.layout(ctx, &dock_item_bc, data, env);
            self.dock_items.set_origin(ctx, Point::new(0.0, 0.0));
        }

        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter();
        for ((pane_location_data, pane_widget, header_widget), pane_data) in panes_iter.zip(data_iter) {
            let pane_expected_size = Size::new(pane_location_data.width, pane_location_data.height);
            let pane_bc = BoxConstraints::new(pane_expected_size, pane_expected_size);
            let pane_size = pane_widget.layout(ctx, &pane_bc, pane_data, env);
            // align to bottom
            pane_widget.set_origin(ctx, Point::new(
                // Shift X by 1 for border
                bc.max().width - pane_location_data.actual_x_pos - pane_location_data.width + 1.0,
                bc.max().height - pane_size.height
            ));
            // Position header
            let header_expected_size = Size::new(pane_location_data.width, HEADER_HEIGHT);
            let header_bc = BoxConstraints::new(header_expected_size, header_expected_size);
            let header_size = header_widget.layout(ctx, &header_bc, pane_data, env);
            // align to top of content
            header_widget.set_origin(ctx, Point::new(
                // Shift x by one for border
                bc.max().width - pane_location_data.actual_x_pos - pane_location_data.width + 1.0,
                bc.max().height - pane_size.height - header_size.height
            ));
        };
        if data.show_dock {
            ctx.window().set_input_region(None);
        } else {
            ctx.window().set_input_region(Some(self.get_input_area()));
        }

        bc.max()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppState, env: &druid::Env) {
        if data.show_dock {
            // Paint background
            let paint_area = ctx.region().bounding_box();
            ctx.fill(paint_area, &env.get(crate::DOCK_BACKGROUND_COLOR_KEY));
            // Paint dock items
            self.dock_items.paint(ctx, data, env);
        }

        self.persistent_items.paint(ctx, data, env);

        let panes_iter = self.panes.iter_mut();
        let data_iter = data.panes.iter();
        for ((_pane_location_data,pane_widget,pane_header_widget), pane_data) in panes_iter.zip(data_iter) {
            // Paint background
            ctx.fill(pane_widget.layout_rect(), &env.get(crate::PANE_BACKGROUND_COLOR_KEY));
            // Draw header background
            ctx.fill(pane_header_widget.layout_rect(), &env.get(crate::PANE_HEADER_COLOR_KEY));

            // Draw pane contents
            pane_widget.paint(ctx, pane_data, env);
            // Draw header
            pane_header_widget.paint(ctx, pane_data, env);

            // Draw border
            let left = pane_widget.layout_rect().x0.min(pane_header_widget.layout_rect().x0);
            let right = pane_widget.layout_rect().x1.min(pane_header_widget.layout_rect().x1);
            let top = pane_header_widget.layout_rect().y0;
            let bottom = pane_widget.layout_rect().y1;
            let border_color = Color::rgba8(0, 0, 0, 100);
            ctx.fill(Rect::new(left - 1.0, top, left, bottom), &border_color);
            ctx.fill(Rect::new(right, top, right + 1.0, bottom), &border_color);
        };
    }
}