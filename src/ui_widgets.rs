use bevy_egui::egui::{self, Color32, Response, Sense, Ui, Vec2, Stroke};

/// A professional circular knob widget with Matrix styling
pub fn knob(ui: &mut Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, diameter: f32) -> Response {
    let desired_size = Vec2::splat(diameter);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    if response.dragged() {
        let delta = response.drag_delta().y - response.drag_delta().x;
        let speed = (*range.end() - *range.start()) / 200.0;
        *value = (*value + delta * speed).clamp(*range.start(), *range.end());
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let center = rect.center();
        let radius = diameter / 2.0;
        let min = *range.start();
        let max = *range.end();
        let t = (*value - min) / (max - min);

        // Standard audio knob: -150 to +150 degrees (270 degree range)
        let start_angle = -150.0_f32.to_radians();
        let end_angle = 150.0_f32.to_radians();
        let current_angle = start_angle + (end_angle - start_angle) * t;

        // Outer ring (background)
        ui.painter().circle_stroke(
            center,
            radius,
            Stroke::new(2.0, Color32::from_rgb(0, 50, 0)),
        );

        // Inner circle (fill)
        ui.painter().circle_filled(
            center,
            radius - 3.0,
            Color32::from_rgb(5, 5, 5),
        );

        // Active arc indicator (simplified as a filled arc)
        // Draw indicator line
        let indicator_length = radius - 5.0;
        let indicator_vector = Vec2::new(current_angle.sin(), -current_angle.cos()) * indicator_length;
        let indicator_end = center + indicator_vector;
        
        ui.painter().line_segment(
            [center, indicator_end],
            Stroke::new(3.0, Color32::from_rgb(0, 255, 0)),
        );

        // Indicator dot at end
        ui.painter().circle_filled(
            indicator_end,
            3.0,
            Color32::from_rgb(0, 255, 0),
        );

        // Center dot
        ui.painter().circle_filled(
            center,
            2.0,
            Color32::from_rgb(0, 100, 0),
        );
    }

    response
}

/// A professional "Cyber" slider with Matrix styling
pub fn cyber_slider(ui: &mut Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>) -> Response {
    let height = 20.0;
    let (rect, mut response) = ui.allocate_at_least(Vec2::new(120.0, height), Sense::click_and_drag());

    if response.dragged() {
        let range_len = *range.end() - *range.start();
        let delta = response.drag_delta().x;
        let delta_value = (delta / rect.width()) * range_len;
        *value = (*value + delta_value).clamp(*range.start(), *range.end());
        response.mark_changed();
    }
    
    // Handle click to set value
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let t = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            let range_len = *range.end() - *range.start();
            *value = *range.start() + t * range_len;
            response.mark_changed();
        }
    }
    
    if ui.is_rect_visible(rect) {
        let t = (*value - *range.start()) / (*range.end() - *range.start());
        
        // Background track
        ui.painter().rect_filled(
            rect,
            2.0,
            Color32::from_rgb(10, 10, 10),
        );
        
        // Border
        ui.painter().rect_stroke(
            rect,
            2.0,
            Stroke::new(1.0, Color32::from_rgb(0, 100, 0)),
        );
        
        // Fill (active portion)
        let mut fill_rect = rect;
        fill_rect.set_width(rect.width() * t);
        ui.painter().rect_filled(
            fill_rect,
            2.0,
            Color32::from_rgba_premultiplied(0, 255, 0, 150),
        );
        
        // Handle/thumb
        let handle_x = rect.left() + rect.width() * t;
        let handle_rect = egui::Rect::from_min_size(
            egui::pos2(handle_x - 4.0, rect.top() + 2.0),
            Vec2::new(8.0, rect.height() - 4.0)
        );
        
        ui.painter().rect_filled(
            handle_rect,
            1.0,
            Color32::from_rgb(0, 255, 0),
        );
        
        // Handle border
        ui.painter().rect_stroke(
            handle_rect,
            1.0,
            Stroke::new(1.0, Color32::from_rgb(255, 255, 255)),
        );
    }
    
    response
}
