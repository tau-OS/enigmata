use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;

use gtk4::gio;
use gtk4::prelude::*;
use libhelium::prelude::*;
use relm4::main_application;
use relm4::prelude::*;
use sourceview5::prelude::ViewExt;
// use sourceview5::prelude::BufferExt;
use sourceview5::prelude::*;
mod shortcuts;
mod ui;

struct MainWindow {
    text: String,
    line: i32,
    column: i32,
    char_count: i32,
    /// The current file the buffer is associated with
    current_file: Option<std::path::PathBuf>,

    /// Hash of the current file, calculated on load
    file_hash: Option<u64>,

    /// The actual text input buffer
    buffer: sourceview5::Buffer,

    search_bar: relm4::component::Connector<ui::search::SearchBar>,

    /// Indicates if the buffer has unsaved changes, AKA "dirty"
    is_dirty: bool,
}

#[derive(Debug)]
pub enum AppMsg {
    /// Emits when the text in the editor changes
    TextChanged(String),
    /// Emits when the cursor position changes
    UpdateCursorPos(i32, i32, i32),

    /// Opens file dialog
    Open,
    /// Set the contents of the buffer, used in conjunction with `LoadBuffer`
    SetBufferData(String),
    /// Save current file to disk
    /// If no file path is set, calls `SaveAs`
    Save,
    /// Save current file to disk with a new name
    /// Calls `SaveBuffer` with the new file path
    SaveAs,
    // SaveContent(String),
    Quit,
    Idk,
    /// Displays about dialog
    About,

    /// Find/Search
    Find,

    // Messages for i/o
    /// Load file to buffer
    LoadBuffer(PathBuf),
    /// Save buffer to file
    SaveBuffer(PathBuf, String),

    SetStyleScheme(sourceview5::StyleScheme),
    SelectStyleScheme,

    /// Set text highlighting language
    SetLanguage(Option<sourceview5::Language>),
}

impl MainWindow {
    fn default_file_name(&self) -> String {
        const UNTITLED: &str = "Untitled.txt";
        self.current_file
            .as_ref()
            .map(|f| f.file_name().and_then(|f| f.to_str()).unwrap_or(UNTITLED))
            .unwrap_or_else(|| UNTITLED)
            .to_string()
    }

    fn guess_language_from_file(&self) -> Option<sourceview5::Language> {
        let langman = sourceview5::LanguageManager::default();

        let file_path = self.current_file.as_ref();

        let l = langman.guess_language(file_path, None);
        println!("Guessing language: {:?}", l);
        l
    }

    /// Hash the data in the current buffer
    ///
    /// Used to determine if the buffer is dirty and requires saving
    fn hash_buffer_data(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        let content = self
            .buffer
            .text(&self.buffer.start_iter(), &self.buffer.end_iter(), true);
        content.hash(&mut hasher);
        hasher.finish()
    }
}

#[relm4::component]
impl SimpleComponent for MainWindow {
    type Init = String;
    type Input = AppMsg;
    type Output = ();

    view! {
        #[root]
        main_window = libhelium::ApplicationWindow {
            set_title: Some("Enigmata"),
            set_icon_name: Some("accessories-text-editor"),
            set_show_menubar: false,
            set_vexpand: false,
            set_decorated: true,

            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::Quit);
                gtk::glib::Propagation::Stop
            },

            set_default_size: (1280, 720),
            #[wrap(Some)]
            set_titlebar = &libhelium::AppBar {
                set_is_compact: true,
                // set_align: gtk::Align::Fill,
                set_vexpand: false,
                set_css_classes: &["app-bar", "vim-status-bar"],
                set_overflow: gtk::Overflow::Visible,
                // set_child = &gtk::Box {
                //     gtk::Label {
                //         set_label: "hiii!!!!",
                //     }
                // }
            },

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                #[name = "overlay"]
                gtk::Overlay {
                    set_hexpand: true,
                    set_vexpand: true,
                    add_overlay: search_bar,

                    #[wrap(Some)]
                    #[name = "main_view"]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::ScrolledWindow {
                            set_vexpand: true,
                            set_hexpand: true,
                            set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Automatic),
                            #[name = "source_view"]
                            sourceview5::View {
                                set_expand: true,
                                set_input_purpose: gtk::InputPurpose::FreeForm,
                                set_wrap_mode: gtk::WrapMode::WordChar,
                                set_show_line_numbers: true,
                                set_highlight_current_line: true,
                                set_monospace: true,
                                set_background_pattern: sourceview5::BackgroundPatternType::Grid,
                                // set_extra_menu: Some(&{
                                //     let menu: gtk4::gio::MenuModel = build_menu().into();
                                //     menu
                                // }),
                                // 
                                set_widget_name: "source_view",
                                set_accessible_role: gtk::AccessibleRole::TextBox,
                            },
                        },
                    }, // gtk::Overlay

                }, // gtk::Box 
                #[name = "status_bar"]
                libhelium::BottomBar {
                    set_css_classes: &["compact"],
                    // set_align: gtk::Align::BaselineFill,
                    set_expand: false,
                    // set_: asdasd,
                    //
                    set_menu_model: &gtk4::gio::MenuModel::from(build_menu()),
                    #[watch]
                    set_title: &format!("{}{}",
                        model.current_file.clone().map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Untitled".to_string()),
                        if model.is_dirty { "*" } else { "" }
                    ),
                    #[watch]
                    set_description: &format!("Line {}, Column {} | Characters: {}", model.line, model.column, model.char_count),
                    set_widget_name: "status_bar",
                    #[name = "open_button_shortcut"]
                    prepend_button[libhelium::BottomBarPosition::Left] = &libhelium::Button {
                        
                        set_is_pill: true,
                        // set_is_tint: true,
                        set_css_classes: &["circular"],
                        set_margin_horizontal: 8,
                        set_tooltip_text: Some("Open file..."),
                        // set_label: "Open",
                        set_icon_name: "document-open-symbolic",
                        set_is_iconic: true,
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::Open);
                        },
                    },
                    
                    #[name = "search_button_shortcut"]
                    append_button[libhelium::BottomBarPosition::Right] = &libhelium::Button {
                        // set_is_pill: true,
                        // set_is_tint: true,
                        set_css_classes: &["circular"],
                        set_tooltip_text: Some("Search..."),
                        set_margin_horizontal: 8,
                        // set_label: "Search",
                        set_icon_name: "edit-find-symbolic",
                        set_is_iconic: true,
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::Find);
                        },
                    },
                }, // libhelium::BottomBar
            },
        }
    }

    fn init(
        text: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        use sourceview5::prelude::BufferExt;
        let style_scheme = sourceview5::StyleSchemeManager::default().scheme("Adwaita-dark");
        let buffer = sourceview5::Buffer::new(None);
        buffer.set_style_scheme(style_scheme.as_ref());

        let mut model = MainWindow {
            text,
            line: 1,
            column: 1,
            char_count: 0,
            current_file: None,
            search_bar: ui::search::SearchBar::builder().launch(buffer.clone()),
            buffer: buffer.clone(),
            is_dirty: false,
            file_hash: None,
        };

        model.search_bar.detach_runtime();

        let search_bar = model.search_bar.widget();
        let buffer = &model.buffer;

        let widgets = view_output!();
        widgets.source_view.set_buffer(Some(&model.buffer));

        {
            let sender_clone = sender.clone();
            buffer.connect_changed(move |buffer| {
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let char_count = text.as_str().chars().count() as i32;

                let cursor_iter = buffer.iter_at_mark(&buffer.get_insert());
                let line = cursor_iter.line() + 1;
                let column = cursor_iter.line_offset() + 1;

                sender_clone.input(AppMsg::UpdateCursorPos(line, column, char_count));
                sender_clone.input(AppMsg::TextChanged(text.to_owned().into()));
            });
        }

        {
            let sender_clone = sender.clone();
            buffer.connect_mark_set(move |buffer, iter, mark| {
                if mark.name().as_deref() == Some("insert") {
                    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                    let char_count = text.as_str().chars().count() as i32;

                    let line = iter.line() + 1;
                    let column = iter.line_offset() + 1;

                    sender_clone.input(AppMsg::UpdateCursorPos(line, column, char_count));
                }
            });
        }

        let shortcutman = shortcuts::ShortcutManager::default();
        crate::gen_shortcut!(shortcutman sender);

        // Actions setup
        shortcut!("<Primary>o" => Open);
        shortcut!("<Primary>s" => Save);
        shortcut!("<Primary><Shift>s" => SaveAs);
        shortcut!("<Primary>q|<Primary>w" => Quit);
        shortcut!("<Primary>equal" => ZoomIn => {
            println!("Zoom in");
        });
        shortcut!("<Primary>minus" => ZoomOut => {
            println!("Zoom out");
        });
        shortcut!("<Primary>f" => Find);

        let sender_idk = sender.clone();
        let action_idk = gtk4::gio::SimpleAction::new("idk", None);
        action_idk.connect_activate(move |_, _| {
            sender_idk.input(AppMsg::Idk);
        });
        shortcutman.actions.add_action(&action_idk);

        let sender_about = sender.clone();
        let action_about = gtk4::gio::SimpleAction::new("about", None);
        action_about.connect_activate(move |_, _| {
            sender_about.input(AppMsg::About);
        });
        shortcutman.actions.add_action(&action_about);

        let sender_selectstylescheme = sender.clone();
        let action_selectstylescheme = gtk4::gio::SimpleAction::new("selectstylescheme", None);
        action_selectstylescheme.connect_activate(move |_, _| {
            sender_selectstylescheme.input(AppMsg::SelectStyleScheme);
        });
        shortcutman.actions.add_action(&action_selectstylescheme);

        widgets.source_view.add_controller(shortcutman.shortcut_ctl);
        widgets
            .main_window
            .insert_action_group("app", Some(&shortcutman.actions));

        main_application().connect_open(move |app, files, _| {
            app.activate();
            if let Some(file_path) = files.first().and_then(|file| file.path()) {
                sender.input(AppMsg::LoadBuffer(file_path));
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Find => {
                self.search_bar
                    .sender()
                    .send(ui::search::SearchBarMsg::Trigger)
                    .unwrap();
            }
            AppMsg::TextChanged(text) => {
                self.text = text;
                // Mark buffer as dirty if we hashed the current buffer
                // in memory, and the hash doesn't match the currently
                // stored one
                if let Some(stored_hash) = self.file_hash {
                    // println!("Calculating hash...");
                    // println!("Stored hash: {}", stored_hash);
                    // println!("Current hash: {}", self.hash_buffer_data());

                    // todo:
                    let current_hash = self.hash_buffer_data();
                    self.is_dirty = stored_hash != current_hash;
                } else {
                    self.is_dirty = true;
                }
            }
            AppMsg::UpdateCursorPos(line, column, char_count) => {
                self.line = line;
                self.column = column;
                self.char_count = char_count;
            }
            // Set content to buffer
            AppMsg::SetBufferData(content) => {
                self.text = content.clone();
                // hash the buffer
                self.buffer.set_text(&content);
                // hash the buffer
                self.file_hash = Some(self.hash_buffer_data());
            }
            // Load file to buffer
            AppMsg::LoadBuffer(file_path) => {
                let Ok(content) = std::fs::read(&file_path) else {
                    return;
                };
                let content = String::from_utf8_lossy(&content).into_owned();
                self.current_file = Some(file_path.clone());
                sender.input(AppMsg::SetBufferData(content));
                println!("File opened successfully: {}", file_path.display());
                // Mark buffer as clean until changes are made
                self.is_dirty = false;
                // Set text highlighting
                let lang = self.guess_language_from_file();
                sender.input(AppMsg::SetLanguage(lang));
            }
            AppMsg::Open => {
                let file_filter = gtk::FileFilter::new();
                file_filter.add_mime_type("text/*");
                file_filter.set_name(Some("Text files"));
                file_filter.add_pattern("*.txt");

                // todo: switch to GTK file dialog
                let file_chooser = gtk::FileDialog::builder()
                    // .filters(&[&file_filter])
                    // .filter(&file_filter)
                    // .action(gtk::FileChooserAction::Open)
                    // .name("Open File")
                    // .modal(true)
                    .title("Open File")
                    .build();

                // let sender = sender.clone();
                file_chooser.open(
                    None::<&gtk::Window>,
                    None::<&gio::Cancellable>,
                    move |res| {
                        if let Ok(file) = res {
                            if let Some(file_path) = file.path() {
                                sender.input(AppMsg::LoadBuffer(file_path));
                            }
                        }
                    },
                );
            }

            AppMsg::Save => {
                if let Some(file_path) = &self.current_file {
                    let content = self
                        .buffer
                        .text(&self.buffer.start_iter(), &self.buffer.end_iter(), false)
                        .to_string();
                    sender.input(AppMsg::SaveBuffer(file_path.clone(), content));
                } else {
                    sender.input(AppMsg::SaveAs);
                }
            }

            AppMsg::SaveAs => {
                let file_filter = gtk::FileFilter::new();
                file_filter.add_mime_type("text/*");
                file_filter.set_name(Some("Text files"));
                file_filter.add_pattern("*.txt");

                let file_chooser = gtk::FileDialog::builder()
                    // .filters(&[&file_filter])
                    // .filter(&file_filter)
                    // .action(gtk::FileChooserAction::Save)
                    // .name("Save File")
                    // .modal(true)
                    .title("Save as...")
                    .initial_name(&self.default_file_name())
                    .build();

                // let sender = sender.clone();
                let model_buffer = self.buffer.clone();
                file_chooser.save(
                    None::<&gtk::Window>,
                    None::<&gio::Cancellable>,
                    move |res| {
                        if let Ok(file) = res {
                            if let Some(file_path) = file.path() {
                                let content = model_buffer
                                    .text(
                                        &model_buffer.start_iter(),
                                        &model_buffer.end_iter(),
                                        false,
                                    )
                                    .to_string();
                                sender.input(AppMsg::SaveBuffer(file_path, content));
                            }
                        }
                    },
                );
            }
            // AppMsg::SaveContent(content) => {
            //     if let Some(file_path) = &self.current_file {
            //         match std::fs::write(file_path, &content) {
            //             Ok(_) => {
            //                 println!("File saved successfully at: {}", file_path.display());
            //             }
            //             Err(e) => {
            //                 println!("Error saving file: {}", e);
            //             }
            //         }
            //     }
            // }
            AppMsg::SetLanguage(lang) => {
                self.buffer.set_language(lang.as_ref());
            }
            AppMsg::SaveBuffer(file_path, content) => {
                println!("Saving buffer to file: {}", file_path.display());
                match std::fs::write(&file_path, &content) {
                    Ok(_) => {
                        println!("File saved successfully at: {}", file_path.display());
                    }
                    Err(e) => {
                        println!("Error saving file: {}", e);
                    }
                }
            }
            AppMsg::Quit => {
                println!("Quitting...");

                println!("Dirty buffer: {:?}", self.is_dirty);

                if self.is_dirty {
                    let alert = gtk::AlertDialog::builder()
                        // .title("Unsaved changes")
                        .message("Unsaved changes")
                        .detail("You have unsaved changes. Do you want to save before quitting?")
                        // .buttons(&[
                        //     ("Save", gtk::ResponseType::Yes),
                        //     ("Don't Save", gtk::ResponseType::No),
                        //     ("Cancel", gtk::ResponseType::Cancel),
                        // ])
                        // .buttons(gtk::ButtonsType::YesNoCancel)
                        .buttons(vec!["Close without saving", "Cancel", "Save"])
                        .cancel_button(2)
                        .default_button(3)
                        .modal(true)
                        .build();
                    alert.choose(
                        None::<&gtk::Window>,
                        None::<&gio::Cancellable>,
                        move |response| {
                            if let Ok(res) = response {
                                match res {
                                    0 => {
                                        std::process::exit(0);
                                    }
                                    1 => {
                                        // Cancel
                                    }
                                    2 => {
                                        sender.input(AppMsg::Save);
                                    }
                                    _ => {}
                                }
                            }
                        },
                    );
                    // alert.show(None::<&gtk::Window>);
                } else {
                    std::process::exit(0);
                }

                // std::process::exit(0);
            }
            AppMsg::Idk => {
                println!("IDK clicked");
            }

            AppMsg::SetStyleScheme(scheme) => {
                println!("Setting style scheme: {:?}", scheme.id());
                self.buffer.set_style_scheme(Some(&scheme));
            }
            AppMsg::SelectStyleScheme => {
                // let style_scheme = sourceview5::StyleSchemeChooserWidget::builder().build();

                relm4::view! {
                    // style_scheme = sourceview5::StyleSchemeChooserButton {
                    // },
                    window = gtk::Window {
                        // set_title: "Select Style Scheme",
                        set_default_size: (400, 400),
                        // #[wrap(Some)]
                        // sourceview5::StyleSchemeChooserButton {
                        //     connect_style_scheme_notify[sender] => move |button| {
                        //         let scheme = button.style_scheme();
                        //         sender.input(AppMsg::SetStyleScheme(scheme));
                        //     }
                        // },
                        //
                        gtk::ScrolledWindow {
                            set_vexpand: true,
                            set_hexpand: true,
                            set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Automatic),
                            sourceview5::StyleSchemeChooserWidget {
                                // set_style_scheme: self.buffer.style_scheme().unwrap(),
                                connect_style_scheme_notify[sender] => move |button| {
                                    let scheme = button.style_scheme();
                                    sender.input(AppMsg::SetStyleScheme(scheme));
                                }
                            },
                        },

                    },

                    // style_chooser =
                }

                window.present();
                // sender.input(AppMsg::SetStyleScheme(style_scheme));
            }
            AppMsg::About => {
                relm4::view! {
                    about = libhelium::AboutWindow {
                        set_app_name: "Enigmata",
                        set_app_id: APP_ID,
                        set_version: env!("CARGO_PKG_VERSION"),
                        set_developer_names: &[
                            "Eri Ishihara <eri@nijika.dev",
                            "Cappy Ishihara <cappy@fyralabs.com>",
                        ],
                        set_copyright_year: 2025,
                        set_modal: true,
                        // #[wrap(Some)]
                        set_issue_url: Some("https://github.com/tau-OS/enigmata/issues"),
                        set_more_info_url: Some("https://github.com/tau-OS/enigmata"),
                        set_icon: "accessories-text-editor",
                        set_license: libhelium::AboutWindowLicenses::Gplv3,
                    }
                };

                about.present();
            }
        }
    }
}

fn build_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    let enigmata_menu = gio::Menu::new();
    let file_menu = gio::Menu::new();
    let help_menu = gio::Menu::new();

    file_menu.append_item(&gio::MenuItem::new(Some("Open"), Some("app.open")));
    file_menu.append_item(&gio::MenuItem::new(Some("Save"), Some("app.save")));
    file_menu.append_item(&gio::MenuItem::new(Some("Save As"), Some("app.saveas")));
    enigmata_menu.append_item(&gio::MenuItem::new(Some("Exit"), Some("app.exit")));
    enigmata_menu.append_item(&gio::MenuItem::new(
        Some("Set Style Scheme"),
        Some("app.selectstylescheme"),
    ));

    // file_menu.append_item(&gio::MenuItem::new(Some("Nothing yet..."), Some("app.idk")));

    help_menu.append_item(&gio::MenuItem::new(Some("About"), Some("app.about")));

    menu.append_submenu(Some("File"), &file_menu);
    menu.append_submenu(Some("View"), &enigmata_menu);
    menu.append_submenu(Some("Help"), &help_menu);

    menu
}

const APP_ID: &str = "com.fyralabs.Enigmata";
use gtk4::glib::translate::FromGlibPtrNone;

fn main() {
    let happ = libhelium::Application::builder()
        .application_id(APP_ID)
        .flags(libhelium::gtk::gio::ApplicationFlags::HANDLES_OPEN)
        .default_accent_color(unsafe {
            &libhelium::RGBColor::from_glib_none(std::ptr::from_mut(
                &mut libhelium::ffi::HeRGBColor {
                    r: 0.0,
                    g: 7.0,
                    b: 143.0,
                },
            ))
        })
        .build();

    happ.connect_open(move |_app, files, _| {
        // let sender = app.sen
        for file in files {
            if let Some(file) = file.path() {
                println!("Opening file: {}", file.display());
            }
        }
    });

    let app = RelmApp::from_app(happ);
    app.allow_multiple_instances(true);
    app.run::<MainWindow>("".to_string());
}
