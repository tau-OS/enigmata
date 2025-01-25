use relm4::gtk;
use relm4::gtk::prelude::*;
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
            set_halign: gtk::Align::Fill,
            set_valign: gtk::Align::End,
            set_overflow: gtk::Overflow::Visible,
            #[local_ref] find_revealer ->
            gtk::Revealer {
                set_transition_duration: 1000,
                set_transition_type: gtk::RevealerTransitionType::SlideUp,

                #[local_ref] search_entry ->
                gtk::SearchEntry {
                    connect_search_changed[sender] => move |search_entry| {
                        let query = search_entry.text();
                        sender.input(SearchBarMsg::UpdateSearchQuery(query.into()));
                    },
                }
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
            SearchBarMsg::UpdateSearchQuery(query) => {
                self.search_context
                    .settings()
                    .set_search_text((!query.is_empty()).then_some(&query));
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
