use eframe::{
    egui::{CentralPanel, Separator, Ui},
    epaint::Color32,
    App,
};
use serde_yaml::mapping::Entry;

pub const PADDING: f32 = 5.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const RED: Color32 = Color32::from_rgb(255, 0, 0);

pub struct EntryView;

impl EntryView {
    fn render_header(ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("PictureManager");
        });
        ui.add_space(PADDING);

        let sep = Separator::default().spacing(20.);
        ui.add(sep);
    }
}

impl Default for EntryView {
    fn default() -> Self {
        Self {}
    }
}

impl App for EntryView {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            Self::render_header(ui);
        });
    }
}
