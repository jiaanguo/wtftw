use crate::core::workspaces::Workspaces;
use crate::handlers::default::{
    exit, move_window_to_workspace, restart, start_launcher, start_terminal, switch_to_workspace,
};
use crate::handlers::{KeyHandler, LogHook, ManageHook, MouseHandler, StartupHook};
use crate::layout::{
    AvoidStrutsLayout, BinarySpacePartition, Direction, FullLayout, GapLayout, Layout,
    LayoutCollection, LayoutMessage, MirrorLayout, NoBordersLayout,
};
use crate::window_manager::WindowManager;
use crate::window_system::{
    KeyCommand, KeyModifiers, MouseButton, MouseCommand, Window, WindowSystem, BUTTON1, BUTTON3,
};

use dylib::DynamicLibrary;
// use log::{debug, error, info};

use std::borrow::ToOwned;
use std::collections::BTreeMap;
// use std::error::Error;
// use std::fs::metadata;
// use std::fs::File;
// use std::fs::{create_dir_all, read_dir};
// use std::io::Write;
// use std::mem;
use std::ops::Deref;
// use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
// use std::process::Command;
use std::rc::Rc;
use std::sync::RwLock;
// use std::thread::spawn;

pub struct GeneralConfig {
    /// Whether focus follows mouse movements or
    /// only click events and keyboard movements.
    pub focus_follows_mouse: bool,
    /// Border color for focused windows.
    pub focus_border_color: u32,
    /// Border color for unfocused windows.
    pub border_color: u32,
    /// Border width. This is the same for both, focused and unfocused.
    pub border_width: u32,
    /// Default terminal to start
    pub terminal: (String, String),
    /// Keybind for the terminal
    /// Path to the logfile
    pub logfile: String,
    /// Default tags for workspaces
    pub tags: Vec<String>,
    /// Default launcher application
    pub launcher: String,
    pub mod_mask: KeyModifiers,
    pub pipes: Vec<Rc<RwLock<Child>>>,
    pub layout: Box<dyn Layout>,
}

impl Clone for GeneralConfig {
    fn clone(&self) -> GeneralConfig {
        GeneralConfig {
            focus_follows_mouse: self.focus_follows_mouse,
            focus_border_color: self.focus_border_color,
            border_color: self.border_color,
            border_width: self.border_width,
            terminal: self.terminal.clone(),
            logfile: self.logfile.clone(),
            tags: self.tags.clone(),
            launcher: self.launcher.clone(),
            mod_mask: self.mod_mask.clone(),
            pipes: self.pipes.clone(),
            layout: self.layout.copy(),
        }
    }
}

pub struct InternalConfig {
    pub library: Option<DynamicLibrary>,
    pub key_handlers: BTreeMap<KeyCommand, KeyHandler>,
    pub mouse_handlers: BTreeMap<MouseCommand, MouseHandler>,
    pub manage_hook: ManageHook,
    pub startup_hook: StartupHook,
    pub loghook: Option<LogHook>,
    pub wtftw_dir: String,
}

impl InternalConfig {
    pub fn new(manage_hook: ManageHook, startup_hook: StartupHook, home: String) -> InternalConfig {
        InternalConfig {
            library: None,
            key_handlers: BTreeMap::new(),
            mouse_handlers: BTreeMap::new(),
            manage_hook: manage_hook,
            startup_hook: startup_hook,
            loghook: None,
            wtftw_dir: format!("{}/.wtftw", home),
        }
    }
}

/// Common configuration options for the window manager.
pub struct Config {
    pub general: GeneralConfig,
    pub internal: InternalConfig,
}

impl Config {
    /// Create the Config from a json file
    pub fn initialize() -> Config {
        let home = dirs::home_dir()
            .unwrap_or(PathBuf::from("./"))
            .into_os_string()
            .into_string()
            .unwrap();
        // Default version of the config, for fallback
        let general_config = GeneralConfig {
            focus_follows_mouse: true,
            focus_border_color: 0xebebeb,
            border_color: 0x404040,
            border_width: 2,
            mod_mask: KeyModifiers::MOD1MASK,
            terminal: (String::from("alacrity"), String::from("")),
            logfile: format!("{}/.wtftw.log", home),
            tags: vec![
                "1: term".to_owned(),
                "2: web".to_owned(),
                "3: code".to_owned(),
                "4: media".to_owned(),
            ],
            launcher: "rofi".to_owned(),
            pipes: Vec::new(),
            layout: LayoutCollection::new(vec![
                GapLayout::new(
                    8,
                    AvoidStrutsLayout::new(
                        vec![Direction::Up, Direction::Down],
                        BinarySpacePartition::new(),
                    ),
                ),
                GapLayout::new(
                    8,
                    AvoidStrutsLayout::new(
                        vec![Direction::Up, Direction::Down],
                        MirrorLayout::new(BinarySpacePartition::new()),
                    ),
                ),
                NoBordersLayout::new(Box::new(FullLayout)),
            ]),
        };

        let internal_config = InternalConfig::new(
            Box::new(move |a, _, _| a.clone()),
            Box::new(move |a, _, _| a.clone()),
            //Box::new(Config::default_manage_hook),
            //Box::new(Config::default_startup_hook),
            home,
        );

        Config {
            general: general_config,
            internal: internal_config,
        }
    }

    pub fn default_manage_hook(m: Workspaces, _: Rc<dyn WindowSystem>, _: Window) -> Workspaces {
        m
    }

    pub fn default_startup_hook(
        m: WindowManager,
        _: Rc<dyn WindowSystem>,
        _: &Config,
    ) -> WindowManager {
        m
    }

    pub fn default_configuration(&mut self, w: &dyn WindowSystem) {
        let mod_mask = self.general.mod_mask.clone();

        // Some standard key handlers for starting, restarting, etc.
        self.add_key_handler(
            w.get_keycode_from_string("q"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, ws, c| exit(m, ws, c)),
        );
        self.add_key_handler(
            w.get_keycode_from_string("q"),
            mod_mask,
            Box::new(|m, ws, c| restart(m, ws, c)),
        );
        self.add_key_handler(
            w.get_keycode_from_string("Return"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, ws, c| start_terminal(m, ws, c)),
        );
        self.add_key_handler(
            w.get_keycode_from_string("p"),
            mod_mask,
            Box::new(|m, ws, c| start_launcher(m, ws, c)),
        );

        // Focus and window movement
        self.add_key_handler(
            w.get_keycode_from_string("j"),
            mod_mask,
            Box::new(|m, w, c| m.windows(w.deref(), c, &|x| x.focus_down())),
        );
        self.add_key_handler(
            w.get_keycode_from_string("k"),
            mod_mask,
            Box::new(|m, w, c| m.windows(w.deref(), c, &|x| x.focus_up())),
        );
        self.add_key_handler(
            w.get_keycode_from_string("j"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, w, c| m.windows(w.deref(), c, &|x| x.swap_down())),
        );
        self.add_key_handler(
            w.get_keycode_from_string("k"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, w, c| m.windows(w.deref(), c, &|x| x.swap_up())),
        );
        self.add_key_handler(
            w.get_keycode_from_string("Return"),
            mod_mask,
            Box::new(|m, w, c| m.windows(w.deref(), c, &|x| x.swap_master())),
        );
        self.add_key_handler(
            w.get_keycode_from_string("c"),
            mod_mask,
            Box::new(|m, w, c| {
                m.kill_window(w.deref())
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("t"),
            mod_mask,
            Box::new(|m, w, c| match m.workspaces.peek() {
                Some(window) => m.windows(w.deref(), c, &|x| x.sink(window)),
                None => m.clone(),
            }),
        );

        // Layout messages
        self.add_key_handler(
            w.get_keycode_from_string("h"),
            mod_mask,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::Decrease, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("l"),
            mod_mask,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::Increase, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("z"),
            mod_mask,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::DecreaseSlave, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("a"),
            mod_mask,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::IncreaseSlave, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("x"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::IncreaseGap, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("s"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::DecreaseGap, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("comma"),
            mod_mask,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::IncreaseMaster, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("period"),
            mod_mask,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::DecreaseMaster, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("space"),
            mod_mask,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::Next, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("a"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::Prev, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("r"),
            mod_mask,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::TreeRotate, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("s"),
            mod_mask,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::TreeSwap, w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("u"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(
                    LayoutMessage::TreeExpandTowards(Direction::Left),
                    w.deref(),
                    c,
                )
                .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("p"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(
                    LayoutMessage::TreeExpandTowards(Direction::Right),
                    w.deref(),
                    c,
                )
                .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("i"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(
                    LayoutMessage::TreeExpandTowards(Direction::Down),
                    w.deref(),
                    c,
                )
                .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("o"),
            mod_mask | KeyModifiers::SHIFTMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(
                    LayoutMessage::TreeExpandTowards(Direction::Up),
                    w.deref(),
                    c,
                )
                .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("u"),
            mod_mask | KeyModifiers::CONTROLMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::TreeShrinkFrom(Direction::Left), w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("p"),
            mod_mask | KeyModifiers::CONTROLMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(
                    LayoutMessage::TreeShrinkFrom(Direction::Right),
                    w.deref(),
                    c,
                )
                .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("i"),
            mod_mask | KeyModifiers::CONTROLMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::TreeShrinkFrom(Direction::Down), w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        self.add_key_handler(
            w.get_keycode_from_string("o"),
            mod_mask | KeyModifiers::CONTROLMASK,
            Box::new(|m, w, c| {
                m.send_layout_message(LayoutMessage::TreeShrinkFrom(Direction::Up), w.deref(), c)
                    .windows(w.deref(), c, &|x| x.clone())
            }),
        );
        // Workspace switching and moving
        for i in 1usize..10 {
            self.add_key_handler(
                w.get_keycode_from_string(&i.to_string()),
                mod_mask,
                Box::new(move |m, w, c| switch_to_workspace(m, w, c, i - 1)),
            );

            self.add_key_handler(
                w.get_keycode_from_string(&i.to_string()),
                mod_mask | KeyModifiers::SHIFTMASK,
                Box::new(move |m, w, c| move_window_to_workspace(m, w, c, i - 1)),
            );
        }

        self.add_mouse_handler(
            BUTTON1,
            mod_mask,
            Box::new(|m, w, c, s| {
                m.focus(s, w.deref(), c)
                    .mouse_move_window(w.deref(), c, s)
                    .windows(w.deref(), c, &|x| x.shift_master())
            }),
        );

        self.add_mouse_handler(
            BUTTON3,
            mod_mask,
            Box::new(|m, w, c, s| {
                m.focus(s, w.deref(), c)
                    .mouse_resize_window(w.deref(), c, s)
                    .windows(w.deref(), c, &|x| x.shift_master())
            }),
        );
    }

    pub fn get_mod_mask(&self) -> KeyModifiers {
        self.general.mod_mask.clone()
    }

    pub fn add_key_handler(&mut self, key: u64, mask: KeyModifiers, keyhandler: KeyHandler) {
        self.internal
            .key_handlers
            .insert(KeyCommand::new(key, mask), keyhandler);
    }

    pub fn add_mouse_handler(
        &mut self,
        button: MouseButton,
        mask: KeyModifiers,
        mousehandler: MouseHandler,
    ) {
        self.internal
            .mouse_handlers
            .insert(MouseCommand::new(button, mask), mousehandler);
    }

    pub fn set_manage_hook(&mut self, hook: ManageHook) {
        self.internal.manage_hook = hook;
    }

    pub fn set_log_hook(&mut self, hook: LogHook) {
        self.internal.loghook = Some(hook);
    }

    //     pub fn compile(&self) -> bool {
    //         info!("updating dependencies");
    //         Command::new("cargo")
    //             .current_dir(&Path::new(&self.internal.wtftw_dir.clone()))
    //             .arg("update")
    //             .env("RUST_LOG", "none")
    //             .output()
    //             .unwrap();
    //         info!("compiling config module");
    //         let output = Command::new("cargo")
    //             .current_dir(&Path::new(&self.internal.wtftw_dir.clone()))
    //             .arg("build") //.arg("--release")
    //             .env("RUST_LOG", "none")
    //             .output();

    //         match output {
    //             Ok(o) => {
    //                 if o.status.success() {
    //                     info!("config module compiled");
    //                     true
    //                 } else {
    //                     error!("error compiling config module");

    //                     spawn(move || {
    //                         Command::new("xmessage").arg("\"error compiling config module. run 'cargo build' in ~/.wtftw to get more info.\"").spawn().unwrap();
    //                     });
    //                     false
    //                 }
    //             }
    //             Err(err) => {
    //                 error!("error compiling config module");
    //                 spawn(move || {
    //                     Command::new("xmessage")
    //                         .arg(err.description())
    //                         .spawn()
    //                         .unwrap();
    //                 });
    //                 false
    //             }
    //         }
    //     }

    //     pub fn call(&mut self, m: &mut WindowManager, w: &dyn WindowSystem) {
    //         debug!("looking for config module");
    //         let mut contents = read_dir(&Path::new(&format!(
    //             "{}/target/debug",
    //             self.internal.wtftw_dir.clone()
    //         )))
    //         .unwrap();
    //         let libname = contents.find(|x| match x {
    //             &Ok(ref y) => y
    //                 .path()
    //                 .into_os_string()
    //                 .as_os_str()
    //                 .to_str()
    //                 .unwrap()
    //                 .contains("libconfig.so"),
    //             &Err(_) => false,
    //         });

    //         if let Ok(lib) = DynamicLibrary::open(Some(&Path::new(
    //             &libname
    //                 .unwrap()
    //                 .unwrap()
    //                 .path()
    //                 .as_os_str()
    //                 .to_str()
    //                 .unwrap(),
    //         ))) {
    //             unsafe {
    //                 if let Ok(symbol) = lib.symbol("configure") {
    //                     let result = mem::transmute::<
    //                         *mut u8,
    //                         extern "C" fn(&mut WindowManager, &dyn WindowSystem, &mut Config),
    //                     >(symbol);

    //                     self.internal.library = Some(lib);
    //                     result(m, w, self);
    //                 } else {
    //                     error!("Error loading config module")
    //                 }
    //             }
    //         }
    //     }
    // }
}

// fn path_exists(path: &String) -> bool {
//     metadata(path).is_ok()
// }
