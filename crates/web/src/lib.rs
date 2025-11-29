use eframe::egui;
use graphodoc_core::{GraphData, Node, Edge};
use std::collections::HashMap;
use egui::{Color32, Pos2, Vec2, Rect, Stroke};
use std::sync::{Arc, Mutex};

struct SharedState {
    data: Option<GraphData>,
    error: Option<String>,
    status_text: String,
}

pub struct App {
    state: Arc<Mutex<SharedState>>,
    
    nodes_map: HashMap<String, Node>,
    edges: Vec<Edge>,

    // Physics State (Keys are String)
    positions: HashMap<String, Pos2>,
    velocities: HashMap<String, Vec2>,

    sim_running: bool,
    pan: Vec2,
    zoom: f32,
    dragging_node: Option<String>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let state = Arc::new(Mutex::new(SharedState {
            data: None,
            error: None,
            status_text: "Initializing...".to_owned(),
        }));

        // Async Fetch
        let state_clone = state.clone();
        let request = ehttp::Request::get("/api/graph");
        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            let mut shared = state_clone.lock().unwrap();
            if let Ok(response) = result {
                shared.status_text = format!("HTTP {}", response.status);
                if response.status == 200 {
                    match serde_json::from_slice::<GraphData>(&response.bytes) {
                        Ok(graph) => {
                            shared.status_text = format!("Loaded {} nodes", graph.nodes.len());
                            shared.data = Some(graph);
                        }
                        Err(e) => shared.error = Some(format!("JSON Error: {}", e)),
                    }
                }
            } else {
                shared.error = Some("Network Error".to_string());
            }
        });

        Self {
            state,
            nodes_map: HashMap::new(),
            edges: Vec::new(),
            positions: HashMap::new(),
            velocities: HashMap::new(),
            sim_running: true,
            pan: Vec2::ZERO,
            zoom: 1.0,
            dragging_node: None,
        }
    }

    fn update_physics(&mut self, rect: Rect) {
        if !self.sim_running || self.nodes_map.is_empty() { return; }

        let center = rect.center();

        // 0. Initialize positions (Spiral Layout)
        if self.positions.is_empty() {
            let mut i = 0.0f32;
            for node in self.nodes_map.values() {
                // Spread them out more initially (radius multiplier 15.0 -> 20.0)
                let radius = 20.0 * (10.0 + i).sqrt();
                let angle = i * 0.5;
                self.positions.insert(node.id.clone(), center + Vec2::new(radius * angle.cos(), radius * angle.sin()));
                self.velocities.insert(node.id.clone(), Vec2::ZERO);
                i += 1.0;
            }
        }

        let mut forces: HashMap<String, Vec2> = self.nodes_map.keys().map(|id| (id.clone(), Vec2::ZERO)).collect();
        let node_ids: Vec<String> = self.nodes_map.keys().cloned().collect();

        // 1. Repulsion (Nodes push apart)
        for (i, id1) in node_ids.iter().enumerate() {
            if let Some(&p1) = self.positions.get(id1) {
                for id2 in node_ids.iter().skip(i + 1) {
                    if let Some(&p2) = self.positions.get(id2) {
                        let diff = p1 - p2;
                        // FIX: Clamp minimum distance to 10.0 to prevent division by zero/infinity
                        let dist_sq = diff.length_sq().max(100.0);

                        // Optimization: Only calculate if close enough
                        if dist_sq < 50_000.0 {
                            // Lower repulsion strength (2000.0 -> 500.0)
                            let force = diff.normalized() * (500.0 / dist_sq);
                            if let Some(f) = forces.get_mut(id1) { *f += force; }
                            if let Some(f) = forces.get_mut(id2) { *f -= force; }
                        }
                    }
                }
            }
        }

        // 2. Spring Force (Edges pull together)
        for edge in &self.edges {
            if let (Some(&p1), Some(&p2)) = (self.positions.get(&edge.source), self.positions.get(&edge.target)) {
                let delta = p2 - p1;
                let dist = delta.length();
                // Hooke's Law with max stretch limit
                if dist > 0.1 {
                    let force = delta.normalized() * (dist - 50.0) * 0.05;
                    if let Some(f) = forces.get_mut(&edge.source) { *f += force; }
                    if let Some(f) = forces.get_mut(&edge.target) { *f -= force; }
                }
            }
        }

        // 3. Center Gravity (Keep them on screen!)
        for (id, pos) in &self.positions {
            let diff = center - *pos;
            // Stronger centering force (0.015 -> 0.02)
            if let Some(f) = forces.get_mut(id) { *f += diff * 0.02; }
        }

        // 4. Apply Physics with Clamping
        let mut total_ke = 0.0;
        for (id, force) in forces {
            if self.dragging_node.as_ref() == Some(&id) {
                self.velocities.insert(id, Vec2::ZERO);
                continue;
            }

            if let Some(vel) = self.velocities.get_mut(&id) {
                // FIX: Velocity Clamping! Max speed 5.0 pixels/frame
                let mut new_vel = *vel + force;
                if new_vel.length() > 5.0 {
                    new_vel = new_vel.normalized() * 5.0;
                }

                // Higher Friction (0.85 -> 0.60) to stop them from drifting forever
                *vel = new_vel * 0.60;

                if let Some(pos) = self.positions.get_mut(&id) {
                    *pos += *vel;
                }
                total_ke += vel.length_sq();
            }
        }

        // Auto-pause when movement is negligible
        if total_ke < 0.1 { self.sim_running = false; }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // Sync State
        let (status_text, error_opt) = {
            let mut shared = self.state.lock().unwrap();
            if let Some(data) = shared.data.take() {
                self.nodes_map = data.nodes.into_iter().map(|n| (n.id.clone(), n)).collect();
                self.edges = data.edges;
                self.sim_running = true;
            }
            (shared.status_text.clone(), shared.error.clone())
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter().with_clip_rect(rect);

            if ui.rect_contains_pointer(rect) {
                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                if scroll != 0.0 {
                    let old_zoom = self.zoom;
                    self.zoom *= if scroll > 0.0 { 1.1 } else { 0.9 };
                    self.zoom = self.zoom.clamp(0.1, 5.0);
                }
            }

            self.update_physics(rect);

            let to_screen = |pos: Pos2| -> Pos2 {
                rect.center() + (pos - rect.center()) * self.zoom + self.pan
            };

            // Draw Edges
            for edge in &self.edges {
                if let (Some(&p1), Some(&p2)) = (self.positions.get(&edge.source), self.positions.get(&edge.target)) {
                    painter.line_segment([to_screen(p1), to_screen(p2)], Stroke::new(1.0, Color32::from_gray(60)));
                }
            }

            // Draw Nodes
            for node in self.nodes_map.values() {
                if let Some(pos) = self.positions.get(&node.id) {
                    let screen_pos = to_screen(*pos);
                    let interact_rect = Rect::from_center_size(screen_pos, Vec2::splat(10.0));

                    if ui.input(|i| i.pointer.primary_down()) {
                        if let Some(ptr) = ui.input(|i| i.pointer.hover_pos()) {
                            if interact_rect.contains(ptr) && self.dragging_node.is_none() {
                                self.dragging_node = Some(node.id.clone());
                                self.sim_running = true;
                            }
                        }
                    }
                    if ui.input(|i| i.pointer.primary_released()) { self.dragging_node = None; }

                    if self.dragging_node.as_ref() == Some(&node.id) {
                        if let Some(ptr) = ui.input(|i| i.pointer.hover_pos()) {
                            let sim_pos = rect.center() + (ptr - rect.center() - self.pan) / self.zoom;
                            self.positions.insert(node.id.clone(), sim_pos);
                        }
                    }

                    if rect.contains(screen_pos) {
                        let color = match node.kind.as_str() {
                            "Document" => Color32::from_rgb(100, 149, 237),
                            "Concept" => Color32::from_rgb(144, 238, 144),
                            "Tag" => Color32::from_rgb(255, 165, 0),
                            _ => Color32::GRAY,
                        };
                        painter.circle_filled(screen_pos, 5.0 * self.zoom, color);
                    }
                }
            }

            // Overlay
            egui::Area::new("ui_overlay".into()).fixed_pos(rect.min + Vec2::new(10.0, 10.0)).show(ctx, |ui| {
                ui.label(&status_text);
                if let Some(e) = error_opt { ui.colored_label(Color32::RED, e); }
            });
        });
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start_app() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start("the_canvas_id", web_options, Box::new(|cc| Box::new(App::new(cc))))
            .await
            .expect("failed to start");
    });
}
