use relm4::gtk::prelude::*;
use relm4::{gtk, RelmWidgetExt};
use sourceview5::prelude::*;
#[derive(Debug, Default)]
pub struct SearchBar {
    /// Search entry for searching and replacing text
    search_entry: gtk4::SearchEntry,
    find_revealer: gtk4::Revealer,

    /// GTKSourceView search context
    search_context: sourceview5::SearchContext,
    // /// Settings for the search
    // search_settings: sourceview5::SearchSettings,
}

#[derive(Debug, Clone)]
pub enum SearchBarMsg {
    Trigger,

    UpdateSearchQuery(String),
    SetSearchRegex(bool),
    SetSearchCaseSensitive(bool),
    SetWordBoundarySearch(bool),

    ReplaceInBuffer,
    ReplaceAllInBuffer,

    IterateNextMatch,
    IteratePreviousMatch,
}

#[relm4::component(pub)]
impl relm4::SimpleComponent for SearchBar {
    type Init = sourceview5::Buffer;
    type Input = SearchBarMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_halign: gtk::Align::End,
            set_valign: gtk::Align::End,
            set_overflow: gtk::Overflow::Visible,
            #[local_ref] find_revealer ->
            gtk::Revealer {
                set_overflow: gtk::Overflow::Visible,
                // set_
                set_transition_duration: 300,
                set_transition_type: gtk::RevealerTransitionType::SlideUp,


                gtk::Grid {
                    // set_css_classes: &["badge-info", "content-block"],
                    inline_css: "background-color: @surface_bright_bg_color; border-radius: 8px; padding: 8px; drop-shadow: 0 0 8px rgba(0, 0, 0, 0.5);",
                    set_margin_all: 16,
                    // set_row_homogeneous: true,
                    set_orientation: gtk::Orientation::Vertical,

                    // column, row, width, height

                    #[local_ref]
                    attach[0, 0, 1, 1] = search_entry -> gtk::SearchEntry {
                        set_placeholder_text: Some("Search"),
                        connect_search_changed[sender] => move |search_entry| {
                            let query = search_entry.text();
                            
                            sender.input(SearchBarMsg::UpdateSearchQuery(query.into()));
                        },
                    },
                    
                    
                    
                    attach[0, 1, 1, 1] = &gtk::CheckButton {
                        set_label: Some("Regex"),
                        connect_toggled[sender] => move |button| {
                            let active = button.is_active();
                            sender.input(SearchBarMsg::SetSearchRegex(active));
                        },
                    },
                    
                    attach[1, 1, 1, 1] = &gtk::CheckButton {
                        set_label: Some("Case Sensitive"),
                        connect_toggled[sender] => move |button| {
                            let active = button.is_active();
                            sender.input(SearchBarMsg::SetSearchCaseSensitive(active));
                        },
                    },
                    
                    attach[2, 1, 1, 1] = &gtk::CheckButton {
                        set_label: Some("Whole Words"),
                        connect_toggled[sender] => move |button| {
                            let active = button.is_active();
                            sender.input(SearchBarMsg::SetWordBoundarySearch(active));
                        },
                    },
                },
                // layout
            },
        },
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            search_context: sourceview5::SearchContext::new(
                &init,
                Some(&sourceview5::SearchSettings::new()),
            ),
            // search_settings,
            ..Default::default()
        };

        let find_revealer = &model.find_revealer;
        let search_entry = &model.search_entry;
        let widgets = view_output!();
        relm4::ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: relm4::ComponentSender<Self>) {
        match msg {
            SearchBarMsg::Trigger => {
                self.find_revealer
                    .set_reveal_child(!self.find_revealer.is_child_revealed());
                self.find_revealer.activate();
            }
            SearchBarMsg::UpdateSearchQuery(query) => {
                self.search_context
                    .settings()
                    .set_search_text((!query.is_empty()).then_some(&query));
                
                let txt = self.search_context.settings().search_text();
                
                println!("Search query: {:?}", txt);
            }
            SearchBarMsg::SetSearchRegex(opt) => {
                // Set the search to use regex
                self.search_context.settings().set_regex_enabled(opt);
            }
            SearchBarMsg::SetSearchCaseSensitive(opt) => {
                // Set the search to be case sensitive
                self.search_context.settings().set_case_sensitive(opt);
            }
            SearchBarMsg::SetWordBoundarySearch(opt) => {
                // Set the search to match whole words
                self.search_context.settings().set_at_word_boundaries(opt);
            }
            SearchBarMsg::ReplaceInBuffer => {
                // Replace the current match in the buffer
                // self.search_context.replace
                todo!()
            }
            SearchBarMsg::ReplaceAllInBuffer => todo!(),
            SearchBarMsg::IterateNextMatch => {
                todo!()
                // self.search_context.forward(&something)
            }
            SearchBarMsg::IteratePreviousMatch => todo!(),
        }
    }
}
