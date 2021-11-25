#[macro_use]
extern crate penrose;

use std::{fs::File, process::Command};

use penrose::{
    Backward, Config, Forward, Less, More, Result, WindowManager,
    __test_helpers::XConn,
    core::{
        helpers::index_selectors,
        hooks::Hooks,
        layout::{monocle, side_stack},
        Hook, Layout,
    },
    logging_error_handler,
    xcb::new_xcb_backed_window_manager,
};

use simplelog::{LevelFilter, WriteLogger};

use log::error;

const BAR_HEIGHT: usize = 16;

// const FONT: &str = "Nerd Font Inconsolata";

pub struct StartupScript {
    path: String,
}
impl StartupScript {
    pub fn new(s: impl Into<String>) -> Self {
        Self { path: s.into() }
    }
}
impl<X: XConn> Hook<X> for StartupScript {
    fn startup(&mut self, _: &mut WindowManager<X>) -> Result<()> {
        Command::new("sh")
            .arg("-c")
            .arg(&self.path)
            .spawn()
            .map(|_| ())
            .unwrap();

        Ok(())
    }
}

fn main() -> penrose::Result<()> {
    // Initialise the logger (use LevelFilter::Debug to enable debug logging)
    if let Err(e) = WriteLogger::init(
        LevelFilter::Info,
        simplelog::Config::default(),
        File::create("/home/teo/.local/share/xorg/wm.log").unwrap(),
    ) {
        panic!("unable to set log level: {}", e);
    };

    let layouts = vec![
        Layout::new("[ ]", Default::default(), side_stack, 1, 0.55),
        Layout::new("[M]", Default::default(), monocle, 1, 0.55),
    ];

    let config = Config::default()
        .builder()
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        .layouts(layouts)
        .border_px(2)
        .gap_px(0)
        .focused_border("#005577")?
        .bar_height(BAR_HEIGHT as u32)
        .floating_classes(vec!["dmenu", "dunst", "polybar"])
        .build()
        .expect("failed to build config");

    let hooks: Hooks<_> = vec![Box::new(StartupScript::new(
        "/home/teo/.config/polybar/launch.sh",
    ))];

    let key_bindings = gen_keybindings! {
        "M-j" => run_internal!(cycle_client, Forward);
        "M-k" => run_internal!(cycle_client, Backward);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        "M-q" => run_internal!(kill_client);
        "M-Tab" => run_internal!(toggle_workspace);
        "M-grave" => run_internal!(cycle_layout, Forward);
        "M-S-grave" => run_internal!(cycle_layout, Backward);
        "M-i" => run_internal!(update_max_main, More);
        "M-d" => run_internal!(update_max_main, Less);
        "M-l" => run_internal!(update_main_ratio, More);
        "M-h" => run_internal!(update_main_ratio, Less);
        "M-space" => run_external!("dmenu_run");
        "M-Return" => run_external!("alacritty");
        "M-S-q" => run_internal!(exit);

        // screen management
        "M-period" => run_internal!(cycle_screen, Forward);
        "M-comma" => run_internal!(cycle_screen, Backward);
        "M-S-period" => run_internal!(drag_workspace, Forward);
        "M-S-comma" => run_internal!(drag_workspace, Backward);

        map: { "1", "2", "3", "4", "5", "6", "7", "8", "9" } to index_selectors(9) => {
            "M-{}" => focus_workspace (REF);
            "M-S-{}" => client_to_workspace (REF);
        };
    };

    let mut wm = new_xcb_backed_window_manager(config, hooks, logging_error_handler())?;
    if let Err(e) = wm.grab_keys_and_run(key_bindings, map! {}) {
        error!("Failed to grab keys and run: {}", e);
        Err(e)
    } else {
        Ok(())
    }
}
