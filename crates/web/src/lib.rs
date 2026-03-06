use eframe::egui;
use graphodoc_core::{GraphData, Node, Edge, NodeId, NodeKind};
use std::collections::{HashMap, HashSet};
use egui::{Color32, Pos2, Vec2, Rect, Stroke, ScrollArea};
use std::sync::{Arc, Mutex};

// Shared state for the async fetcher
struct SharedState {
    data: Option<GraphData>,
    error: Option<String>,
    status_text: String,
}

pub struct App {
    state: Arc<Mutex<SharedState>>,

    // --- Data Source (Immutable after load) ---
    all_nodes: HashMap<NodeId, Node>,
    all_edges: Vec<Edge>,
    sorted_doc_names: Vec<(NodeId, String)>, // (ID, Name) for the sidebar list

    // --- Local Graph State (Dynamic) ---
    selected_node_id: Option<NodeId>,

    // Physics State (Only for the visible local graph)
    positions: HashMap<NodeId, Pos2>,
    velocities: HashMap<NodeId, Vec2>,
    sim_running: bool,

    // Viewport
    pan: Vec2,
    zoom: f32,
    dragging_node: Option<NodeId>,
    search_query: String, // For filtering the sidebar
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let state = Arc::new(Mutex::new(SharedState {
            data: None,
            error: None,
            status_text: "Loading Graph...".to_owned(),
        }));

        let state_clone = state.clone();
        let request = ehttp::Request::get("/api/graph");

        ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
            let mut shared = state_clone.lock().unwrap();
            match result {
                Ok(response) => {
                    if response.status == 200 {
                        match serde_json::from_slice::<GraphData>(&response.bytes) {
                            Ok(graph) => {
                                shared.status_text = format!("Ready.");
                                shared.data = Some(graph);
                            }
                            Err(e) => shared.error = Some(format!("JSON Error: {}", e)),
                        }
                    } else {
                        shared.error = Some(format!("Server Error: {}", response.status));
                    }
                }
                Err(e) => shared.error = Some(format!("Network Error: {}", e)),
            }
        });

        Self {
            state,
            all_nodes: HashMap::new(),
            all_edges: Vec::new(),
            sorted_doc_names: Vec::new(),
            selected_node_id: None,
            positions: HashMap::new(),
            velocities: HashMap::new(),
            sim_running: false,
            pan: Vec2::ZERO,
            zoom: 1.0,
            dragging_node: None,
            search_query: String::new(),
        }
    }

    /// Resets the physics simulation specifically for the new local cluster
    fn reset_simulation(&mut self, rect: Rect, visible_nodes: &HashSet<NodeId>) {
        self.positions.clear();
        self.velocities.clear();
        self.sim_running = true;
        self.pan = Vec2::ZERO;
        self.zoom = 1.0;

        let center = rect.center();
        let mut i = 0.0f32;

        // Place the selected node explicitly in the center
        if let Some(root_id) = &self.selected_node_id {
            if visible_nodes.contains(root_id) {
                self.positions.insert(root_id.clone(), center);
                self.velocities.insert(root_id.clone(), Vec2::ZERO);
            }
        }

        // Scatter neighbors around it
        for node_id in visible_nodes {
            if Some(node_id) == self.selected_node_id.as_ref() { continue; }

            let radius = 100.0 + (i * 10.0);
            let angle = i * 0.8;
            let pos = center + Vec2::new(radius * angle.cos(), radius * angle.sin());

            self.positions.insert(node_id.clone(), pos);
            self.velocities.insert(node_id.clone(), Vec2::ZERO);
            i += 1.0;
        }
    }

    // FIX 2: Signature changed to accept &[Edge] (owned structs) instead of &[&Edge]
    fn update_physics(&mut self, rect: Rect, visible_nodes: &HashSet<NodeId>, visible_edges: &[Edge]) {
        if !self.sim_running || visible_nodes.is_empty() { return; }

        let center = rect.center();
        let mut forces: HashMap<NodeId, Vec2> = visible_nodes.iter().map(|id| (id.clone(), Vec2::ZERO)).collect();

        // 1. Repulsion (Push apart)
        let nodes_list: Vec<&NodeId> = visible_nodes.iter().collect();
        for (i, &id1) in nodes_list.iter().enumerate() {
            if let Some(&p1) = self.positions.get(id1) {
                for &id2 in nodes_list.iter().skip(i + 1) {
                    if let Some(&p2) = self.positions.get(id2) {
                        let diff = p1 - p2;
                        let dist_sq = diff.length_sq().max(100.0);
                        // Stronger repulsion for local graph clarity
                        let force = diff.normalized() * (3000.0 / dist_sq);
                        if let Some(f) = forces.get_mut(id1) { *f += force; }
                        if let Some(f) = forces.get_mut(id2) { *f -= force; }
                    }
                }
            }
        }

        // 2. Springs (Pull connected)
        for edge in visible_edges {
            if let (Some(&p1), Some(&p2)) = (self.positions.get(&edge.source), self.positions.get(&edge.target)) {
                let delta = p2 - p1;
                let dist = delta.length();
                // Tighter springs for local view
                if dist > 1.0 {
                    let force = delta.normalized() * (dist - 100.0) * 0.04;
                    if let Some(f) = forces.get_mut(&edge.source) { *f += force; }
                    if let Some(f) = forces.get_mut(&edge.target) { *f -= force; }
                }
            }
        }

        // 3. Center Gravity
        for (id, pos) in &self.positions {
            let diff = center - *pos;
            let strength = if Some(id) == self.selected_node_id.as_ref() { 0.05 } else { 0.01 };
            if let Some(f) = forces.get_mut(id) { *f += diff * strength; }
        }

        // 4. Integration
        let mut total_ke = 0.0;
        for (id, force) in forces {
            if self.dragging_node.as_ref() == Some(&id) {
                self.velocities.insert(id.clone(), Vec2::ZERO);
                continue;
            }

            if let Some(vel) = self.velocities.get_mut(&id) {
                let mut new_vel = *vel + force;
                // Clamp max speed
                if new_vel.length() > 10.0 { new_vel = new_vel.normalized() * 10.0; }
                *vel = new_vel * 0.65; // High friction

                if let Some(pos) = self.positions.get_mut(&id) { *pos += *vel; }
                total_ke += vel.length_sq();
            }
        }

        if total_ke < 0.1 { self.sim_running = false; }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sync Data
        {
            let mut shared = self.state.lock().unwrap();
            if let Some(data) = shared.data.take() {
                self.all_nodes = data.nodes.into_iter().map(|n| (n.id.clone(), n)).collect();
                self.all_edges = data.edges;

                // Prepare sorted list for sidebar
                self.sorted_doc_names = self.all_nodes.values()
                    .filter(|n| n.kind == NodeKind::Document)
                    .map(|n| (n.id.clone(), n.name.clone()))
                    .collect();
                self.sorted_doc_names.sort_by(|a, b| a.1.cmp(&b.1));
            }
        }

        // --- SIDEBAR (Left Menu) ---
        egui::SidePanel::left("sidebar_panel").resizable(true).default_width(250.0).show(ctx, |ui| {
            ui.heading("📚 Documents");
            ui.separator();

            ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Search..."));
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                let query = self.search_query.to_lowercase();
                for (id, name) in &self.sorted_doc_names {
                    if !query.is_empty() && !name.to_lowercase().contains(&query) {
                        continue;
                    }

                    let is_selected = self.selected_node_id.as_ref() == Some(id);
                    if ui.selectable_label(is_selected, name).clicked() {
                        self.selected_node_id = Some(id.clone());
                        // Trigger physics reset on new selection
                        self.sim_running = false;
                    }
                }
            });
        });

        // --- MAIN CANVAS (Graph) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter().with_clip_rect(rect);

            // 1. Calculate Local Graph (Filter)
            let mut visible_nodes = HashSet::new();
            // FIX 1: Change to Vec<Edge> to hold owned data, not references
            let mut visible_edges: Vec<Edge> = Vec::new();

            if let Some(root_id) = &self.selected_node_id {
                visible_nodes.insert(root_id.clone());

                // Find connected edges
                for edge in &self.all_edges {
                    if &edge.source == root_id {
                        visible_edges.push(edge.clone()); // CLONE HERE
                        visible_nodes.insert(edge.target.clone());
                    } else if &edge.target == root_id {
                        visible_edges.push(edge.clone()); // CLONE HERE
                        visible_nodes.insert(edge.source.clone());
                    }
                }
            }

            // 2. Initialize Physics if needed (Reset when selection changes)
            let need_reset = self.selected_node_id.is_some() && (
                self.positions.is_empty() ||
                    !self.positions.contains_key(self.selected_node_id.as_ref().unwrap()) ||
                    !self.sim_running // Re-trigger via click flag
            );

            if need_reset && !visible_nodes.is_empty() {
                self.reset_simulation(rect, &visible_nodes);
            }

            // 3. Input Handling
            if ui.rect_contains_pointer(rect) {
                // Zoom
                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                if scroll != 0.0 {
                    let old_zoom = self.zoom;
                    self.zoom *= if scroll > 0.0 { 1.1 } else { 0.9 };
                    self.zoom = self.zoom.clamp(0.2, 3.0);
                    if let Some(ptr) = ui.input(|i| i.pointer.hover_pos()) {
                        let offset = ptr - rect.center() - self.pan;
                        let ratio = self.zoom / old_zoom;
                        self.pan -= offset * (ratio - 1.0);
                    }
                }
                // Pan
                if ui.input(|i| i.pointer.middle_down()) {
                    self.pan += ui.input(|i| i.pointer.delta());
                }
            }

            // 4. Physics Step
            if self.sim_running {
                // Now we pass the owned vector, no lifetime issues with 'self'
                self.update_physics(rect, &visible_nodes, &visible_edges);
                ctx.request_repaint(); // Animation loop
            }

            // 5. Render
            let to_screen = |pos: Pos2| -> Pos2 {
                rect.center() + (pos - rect.center()) * self.zoom + self.pan
            };

            // Draw Edges
            for edge in &visible_edges {
                if let (Some(&p1), Some(&p2)) = (self.positions.get(&edge.source), self.positions.get(&edge.target)) {
                    let s1 = to_screen(p1);
                    let s2 = to_screen(p2);
                    painter.line_segment([s1, s2], Stroke::new(1.5 * self.zoom, Color32::from_gray(80)));
                }
            }

            // Draw Nodes
            for node_id in &visible_nodes {
                if let Some(node) = self.all_nodes.get(node_id) {
                    if let Some(&pos) = self.positions.get(node_id) {
                        let screen_pos = to_screen(pos);
                        let radius = 6.0 * self.zoom;

                        // Dragging
                        let interact_rect = Rect::from_center_size(screen_pos, Vec2::splat(radius * 2.5));
                        if ui.input(|i| i.pointer.primary_down()) {
                            if let Some(ptr) = ui.input(|i| i.pointer.hover_pos()) {
                                if interact_rect.contains(ptr) && self.dragging_node.is_none() {
                                    self.dragging_node = Some(node_id.clone());
                                    self.sim_running = true;
                                }
                            }
                        }
                        if ui.input(|i| i.pointer.primary_released()) { self.dragging_node = None; }

                        if self.dragging_node.as_ref() == Some(node_id) {
                            if let Some(ptr) = ui.input(|i| i.pointer.hover_pos()) {
                                let sim_pos = rect.center() + (ptr - rect.center() - self.pan) / self.zoom;
                                self.positions.insert(node_id.clone(), sim_pos);
                            }
                        }

                        // Colors
                        let color = match node.kind {
                            NodeKind::Document => Color32::from_rgb(100, 149, 237), // Blue
                            NodeKind::Concept => Color32::from_rgb(144, 238, 144), // Green
                            NodeKind::Tag => Color32::from_rgb(255, 165, 0),       // Orange
                            NodeKind::Keyword => Color32::from_rgb(200, 200, 200), // Gray
                        };

                        // Highlight Center Node
                        let final_color = if Some(node_id) == self.selected_node_id.as_ref() {
                            Color32::WHITE
                        } else {
                            color
                        };

                        painter.circle_filled(screen_pos, radius, final_color);

                        // Text Labels (Always show name for local graph nodes)
                        painter.text(
                            screen_pos + Vec2::new(0.0, radius + 4.0),
                            egui::Align2::CENTER_TOP,
                            &node.name,
                            egui::FontId::proportional(12.0 * self.zoom),
                            Color32::WHITE,
                        );
                    }
                }
            }

            // Empty State
            if self.selected_node_id.is_none() {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Select a document from the sidebar to view its connections.",
                    egui::FontId::proportional(20.0),
                    Color32::GRAY,
                );
            }
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
