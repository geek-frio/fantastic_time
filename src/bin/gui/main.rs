use eframe::{egui::Context, run_native, CreationContext};
use entry::EntryView;

mod entry;

fn config_fonts(ctx: &Context) {}

fn main() {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .init();

    let native_opts = Default::default();

    run_native(
        "main_view",
        native_opts,
        Box::new(|ctx: &CreationContext| {
            config_fonts(&ctx.egui_ctx);
            Box::new(EntryView)
        }),
    );
}
