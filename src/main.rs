use gtk4::gio;
use gtk4::prelude::*;
use libhelium::prelude::*;
use relm4::prelude::*;
use sourceview5::prelude::ViewExt;
struct AppModel {
    text: String,
    line: i32,
    column: i32,
    char_count: i32,
    current_file: Option<std::path::PathBuf>,
    buffer: sourceview5::Buffer,
}

#[derive(Debug)]
pub enum AppMsg {
    TextChanged(String),
    CursorMoved(i32, i32, i32),
    Open,
    SetContent(String),
    Save,
    SaveAs,
    SaveContent(String),
    Quit,
    Idk,
    About,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = String;
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = libhelium::ApplicationWindow {
            set_title: Some("Enigmata"),

            set_default_size: (1280, 720),
            #[wrap(Some)]
            set_titlebar = &libhelium::AppBar {
                set_is_compact: true,
                #[watch]
                set_viewsubtitle_label: model.current_file.clone().map(|f| f.to_string_lossy().to_string()).unwrap_or_else(|| "Untitled".to_string()).as_ref(),
            },

            #[wrap(Some)]
            set_child = &gtk::Box  {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                gtk::PopoverMenuBar::from_model(Some(&{
                    let menu: gtk4::gio::MenuModel = build_menu().into();
                    menu
                })) {
                },

                #[name = "source_view"]
                sourceview5::View {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_wrap_mode: gtk::WrapMode::WordChar,
                    set_show_line_numbers: true,
                    set_highlight_current_line: true,
                    set_monospace: true,
                    set_background_pattern: sourceview5::BackgroundPatternType::Grid,
                },
                libhelium::BottomBar {
                    set_halign: gtk::Align::End,
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,

                        #[name = "status_label"]
                        gtk::Label {
                            set_hexpand: true,
                            set_xalign: 1.0,
                            set_text: &format!("Line {}, Column {} | Characters: {}",
                                model.line, model.column, model.char_count),
                            set_margin_end: 10,
                            set_margin_bottom: 5,
                            set_margin_top: 5,
                        }
                    }
                }

            }
        }
    }

    fn init(
        text: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = sourceview5::Buffer::new(None);

        let model = AppModel {
            text,
            line: 1,
            column: 1,
            char_count: 0,
            current_file: None,
            buffer: buffer.clone(), // Store the buffer in model
        };

        let widgets = view_output!();
        let actions = gtk4::gio::SimpleActionGroup::new();

        widgets.source_view.set_buffer(Some(&model.buffer));

        {
            let sender_clone = sender.clone();
            let status_label = widgets.status_label.clone();
            buffer.connect_changed(move |buffer| {
                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                let char_count = text.as_str().chars().count() as i32;

                let cursor_iter = buffer.iter_at_mark(&buffer.get_insert());
                let line = cursor_iter.line() + 1;
                let column = cursor_iter.line_offset() + 1;

                sender_clone.input(AppMsg::CursorMoved(line, column, char_count));

                status_label.set_text(&format!(
                    "Line {}, Column {} | Characters: {}",
                    line, column, char_count
                ));
            });
        }

        {
            let sender_clone = sender.clone();
            let status_label = widgets.status_label.clone();
            buffer.connect_mark_set(move |buffer, iter, mark| {
                if mark.name().as_deref() == Some("insert") {
                    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                    let char_count = text.as_str().chars().count() as i32;

                    let line = iter.line() + 1;
                    let column = iter.line_offset() + 1;

                    sender_clone.input(AppMsg::CursorMoved(line, column, char_count));

                    status_label.set_text(&format!(
                        "Line {}, Column {} | Characters: {}",
                        line, column, char_count
                    ));
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

        widgets
            .main_window
            .insert_action_group("app", Some(&actions));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::TextChanged(text) => {
                self.text = text;
            }
            AppMsg::CursorMoved(line, column, char_count) => {
                self.line = line;
                self.column = column;
                self.char_count = char_count;
            }
            AppMsg::SetContent(content) => {
                self.text = content.clone();
                self.buffer.set_text(&content);
            }

            AppMsg::Open => {
                // todo: switch to GTK file dialog
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("Text files", &["txt"])
                    .add_filter("All files", &["*"])
                    .set_title("Open File")
                    .pick_file()
                {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        self.current_file = Some(file_path.clone());
                        sender.input(AppMsg::SetContent(content));
                        println!("File opened successfully: {}", file_path.display());
                    }
                }
            }

            AppMsg::Save => {
                if let Some(_file_path) = &self.current_file {
                    let content = self
                        .buffer
                        .text(&self.buffer.start_iter(), &self.buffer.end_iter(), false)
                        .to_string();
                    sender.input(AppMsg::SaveContent(content));
                } else {
                    sender.input(AppMsg::SaveAs);
                }
            }

            AppMsg::SaveAs => {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("Text files", &["txt"])
                    .add_filter("All files", &["*"])
                    .set_title("Save As")
                    .save_file()
                {
                    self.current_file = Some(file_path);
                    // Get text directly from the buffer
                    let content = self
                        .buffer
                        .text(&self.buffer.start_iter(), &self.buffer.end_iter(), false)
                        .to_string();
                    sender.input(AppMsg::SaveContent(content));
                }
            }
            AppMsg::SaveContent(content) => {
                if let Some(file_path) = &self.current_file {
                    match std::fs::write(file_path, &content) {
                        Ok(_) => {
                            println!("File saved successfully at: {}", file_path.display());
                        }
                        Err(e) => {
                            println!("Error saving file: {}", e);
                        }
                    }
                }
            }
            AppMsg::Quit => {
                println!("Quitting...");
                std::process::exit(0);
            }
            AppMsg::Idk => {
                println!("IDK clicked");
            }
            AppMsg::About => {
                let about = libhelium::AboutWindow::builder()
                    .app_name("Enigmata")
                    .app_id(APP_ID)
                    .version(env!("CARGO_PKG_VERSION"))
                    .developer_names(
                        vec![
                            "Eri Ishihara <eri@nijika.dev>",
                            "Cappy Ishihara <cappy@fyralabs.com>",
                        ]
                        .into_iter()
                        .map(String::from)
                        .collect::<Vec<_>>(),
                    )
                    // .comments("Yet Another GTK4 Text Editor")
                    .copyright_year(2025)
                    .name("Enigmata")
                    .icon("accessories-text-editor")
                    .modal(true)
                    .build();

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

    enigmata_menu.append_item(&gio::MenuItem::new(Some("Open"), Some("app.open")));
    enigmata_menu.append_item(&gio::MenuItem::new(Some("Save"), Some("app.save")));
    enigmata_menu.append_item(&gio::MenuItem::new(Some("Save As"), Some("app.saveas")));
    enigmata_menu.append_item(&gio::MenuItem::new(Some("Exit"), Some("app.exit")));

    file_menu.append_item(&gio::MenuItem::new(Some("Nothing yet..."), Some("app.idk")));

    help_menu.append_item(&gio::MenuItem::new(Some("About"), Some("app.about")));

    menu.append_submenu(Some("Enigmata"), &enigmata_menu);
    menu.append_submenu(Some("File"), &file_menu);
    menu.append_submenu(Some("Help"), &help_menu);

    menu
}

const APP_ID: &str = "com.fyralabs.enigmata";
use gtk4::glib::translate::FromGlibPtrNone;
fn main() {
    let happ = libhelium::Application::builder()
        .application_id(APP_ID)
        .flags(libhelium::gtk::gio::ApplicationFlags::default())
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

    let app = RelmApp::from_app(happ);
    app.run::<AppModel>("".to_string());
}
