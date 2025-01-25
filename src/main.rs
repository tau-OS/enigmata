use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;

use glib::language_names;
use gtk4::gio;
use gtk4::prelude::*;
use libhelium::prelude::*;
use relm4::prelude::*;
use sourceview5::prelude::ViewExt;
// use sourceview5::prelude::BufferExt;
use sourceview5::prelude::*;
struct AppModel {
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

impl AppModel {
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
impl SimpleComponent for AppModel {
    type Init = String;
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = libhelium::ApplicationWindow {
            set_title: Some("Enigmata"),
            set_icon_name: Some("accessories-text-editor"),
            set_show_menubar: true,

            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::Quit);
                gtk::glib::Propagation::Stop
            },

            set_default_size: (1280, 720),
            #[wrap(Some)]
            set_titlebar = &libhelium::AppBar {
                set_is_compact: true,
                set_show_left_title_buttons: true,
                set_show_right_title_buttons: true,
                set_halign: gtk::Align::BaselineFill,
                
                
                #[wrap(Some)]
                #[name = "viewtitle_widget"]
                set_viewtitle_widget = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::End,
                    set_expand: true,
                    gtk::MenuButton {
                        // set_halign: gtk::Align::End,
                        // set_label: "File",
                        // todo: Separate menus for this part, we don't want to reuse the same menus
                        // Maybe the bottom bar menu can be for file ops, and the title bar can be for other stuff
                        set_icon_name: "open-menu-symbolic",
                        set_menu_model: Some(&{
                            let menu: gtk4::gio::MenuModel = build_menu().into();
                            menu
                        }),
                    },
                },

                // #[watch]
                // set_viewsubtitle_label: model.current_file.clone().map(|f| f.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".to_string()).as_ref(),
            },

            #[wrap(Some)]
            #[name = "main_view"]
            set_child = &gtk::Box  {
                set_orientation: gtk::Orientation::Vertical,
                // set_spacing: 0,

                // gtk::PopoverMenuBar::from_model(Some(&{
                //     let menu: gtk4::gio::MenuModel = build_menu().into();
                //     menu
                // })) {
                //     set_expand: false,
                //     set_: 10,
                // },

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
                        // set_sty: Some(&sourceview5::StyleScheme::new_from_builtin(sourceview5::StyleSchemeName::Classic)),
                    },
                },
                // #[name = "status_bar_test"]
                // libhelium::T
                
                #[name = "status_bar"]
                // XXX: I don't think I'm supposed to use this BottomBar for this...
                libhelium::BottomBar {
                    set_css_classes: &["vim-status-bar"],
                    // set_align: gtk::Align::BaselineFill,
                    set_expand: false,
                    // set_: asdasd,
                    // 
                    set_menu_model: &{
                        let menu: gtk4::gio::MenuModel = build_menu().into();
                        menu
                    },
                    #[watch]
                    set_title: &format!("{}{}",
                        model.current_file.clone().map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Untitled".to_string()),
                        if model.is_dirty { "*" } else { "" }
                    ),
                    #[watch]
                    set_description: &format!("Line {}, Column {} | Characters: {}", model.line, model.column, model.char_count),
                    // set_title: "Test",
                    // set_t
                    
                    
                    #[name = "status_menu"]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 6,
                        set_margin_all: 12,

                        #[name = "open_button_shortcut"]
                        libhelium::Button {
                            set_is_pill: true,
                            set_is_tint: true,
                            set_css_classes: &["app-bar-button", "rounded"],
                            set_tooltip_text: Some("Open file..."),
                            // set_label: "Open",
                            set_icon_name: "document-open-symbolic",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppMsg::Open);
                            },
                        }
                    },
                }
            }
        }
    }

    fn init(
        text: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        use sourceview5::prelude::BufferExt;
        let style_scheme = sourceview5::StyleSchemeManager::default()
            .scheme("classic-dark");
        let buffer = sourceview5::Buffer::new(None);
        buffer.set_style_scheme(style_scheme.as_ref());
        // language.guess_language(filename, content_type)

        let model = AppModel {
            text,
            line: 1,
            column: 1,
            char_count: 0,
            current_file: None,
            buffer: buffer.clone(), // Store the buffer in model
            is_dirty: false,
            file_hash: None,
        };

        let widgets = view_output!();
        let actions = gtk4::gio::SimpleActionGroup::new();

        widgets.source_view.set_buffer(Some(&model.buffer));

        {
            let sender_clone = sender.clone();
            // let status_label = widgets.status_label.clone();
            buffer.connect_changed(move |buffer| {
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let char_count = text.as_str().chars().count() as i32;

                let cursor_iter = buffer.iter_at_mark(&buffer.get_insert());
                let line = cursor_iter.line() + 1;
                let column = cursor_iter.line_offset() + 1;

                sender_clone.input(AppMsg::UpdateCursorPos(line, column, char_count));
                // println!("Text changed: {}", text);
                sender_clone.input(AppMsg::TextChanged(text.to_owned().into()));
                // status_label.set_text(&format!(
                //     "Line {}, Column {} | Characters: {}",
                //     line, column, char_count
                // ));
            });
        }

        {
            let sender_clone = sender.clone();
            // let status_label = widgets.status_label.clone();
            buffer.connect_mark_set(move |buffer, iter, mark| {
                if mark.name().as_deref() == Some("insert") {
                    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                    let char_count = text.as_str().chars().count() as i32;

                    let line = iter.line() + 1;
                    let column = iter.line_offset() + 1;

                    sender_clone.input(AppMsg::UpdateCursorPos(line, column, char_count));

                    // status_label.set_text(&format!(
                    //     "Line {}, Column {} | Characters: {}",
                    //     line, column, char_count
                    // ));
                }
            });
        }

        // Actions setup
        let sender_open = sender.clone();
        let action_open = gtk4::gio::SimpleAction::new("open", None);
        action_open.connect_activate(move |_, _| {
            sender_open.input(AppMsg::Open);
        });
        actions.add_action(&action_open);

        let sender_save = sender.clone();
        let action_save = gtk4::gio::SimpleAction::new("save", None);
        action_save.connect_activate(move |_, _| {
            sender_save.input(AppMsg::Save);
        });
        actions.add_action(&action_save);

        let sender_saveas = sender.clone();
        let action_saveas = gtk4::gio::SimpleAction::new("saveas", None);
        action_saveas.connect_activate(move |_, _| {
            sender_saveas.input(AppMsg::SaveAs);
        });
        actions.add_action(&action_saveas);

        let sender_quit = sender.clone();
        let action_exit = gtk4::gio::SimpleAction::new("exit", None);
        action_exit.connect_activate(move |_, _| {
            sender_quit.input(AppMsg::Quit);
        });
        actions.add_action(&action_exit);

        let sender_idk = sender.clone();
        let action_idk = gtk4::gio::SimpleAction::new("idk", None);
        action_idk.connect_activate(move |_, _| {
            sender_idk.input(AppMsg::Idk);
        });
        actions.add_action(&action_idk);

        let sender_about = sender.clone();
        let action_about = gtk4::gio::SimpleAction::new("about", None);
        action_about.connect_activate(move |_, _| {
            sender_about.input(AppMsg::About);
        });
        actions.add_action(&action_about);
        
        let sender_selectstylescheme = sender.clone();
        let action_selectstylescheme = gtk4::gio::SimpleAction::new("selectstylescheme", None);
        action_selectstylescheme.connect_activate(move |_, _| {
            sender_selectstylescheme.input(AppMsg::SelectStyleScheme);
        });
        actions.add_action(&action_selectstylescheme);

        widgets
            .main_window
            .insert_action_group("app", Some(&actions));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
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
                if let Ok(content) = std::fs::read(&file_path) {
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
                    .title("Save File")
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
                        .cancel_button(3)
                        .modal(true)
                        .build();
                    alert.show(None::<&gtk::Window>);
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
                let style_scheme = sourceview5::StyleSchemeChooserWidget::builder()
                    
                    .build();
                
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
    enigmata_menu.append_item(&gio::MenuItem::new(Some("Set Style Scheme"), Some("app.selectstylescheme")));

    // file_menu.append_item(&gio::MenuItem::new(Some("Nothing yet..."), Some("app.idk")));

    help_menu.append_item(&gio::MenuItem::new(Some("About"), Some("app.about")));

    menu.append_submenu(Some("Enigmata"), &enigmata_menu);
    menu.append_submenu(Some("File"), &file_menu);
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
    app.run::<AppModel>("".to_string());
}
