// Release builds on Windows: no console window behind the GUI.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! totem-viewer: a small always-on-top window that shows the TOTEM's active
//! layer live, talking to the RMK firmware over Rynk (USB).

mod app;
mod config;
mod device;
mod labels;

use eframe::egui;

const USAGE: &str = "\
totem-viewer – live layer view for the TOTEM (RMK + Rynk over USB)

USAGE:
    totem-viewer [--config <keyboard.toml>] [--no-top]

OPTIONS:
    --config <path>   Read layer names from this keyboard.toml instead of the
                      one bundled at build time.
    --no-top          Don't keep the window above other windows.
    -h, --help        Show this help.";

struct Args {
    config: Option<String>,
    always_on_top: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        config: None,
        always_on_top: true,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--config" => args.config = Some(it.next().ok_or("--config needs a path")?),
            "--no-top" => args.always_on_top = false,
            "-h" | "--help" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}\n\n{USAGE}")),
        }
    }
    Ok(args)
}

fn main() -> eframe::Result {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let layer_names = config::load_layer_names(args.config.as_deref());

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("TOTEM layers")
        .with_inner_size([620.0, 300.0])
        .with_min_inner_size([320.0, 160.0]);
    if args.always_on_top {
        viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnTop);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "totem-viewer",
        options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            let (state, commands) = device::spawn(move || ctx.request_repaint());
            Ok(Box::new(app::ViewerApp::new(state, commands, layer_names)))
        }),
    )
}
