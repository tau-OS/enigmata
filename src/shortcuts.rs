use gio::prelude::*;

#[derive(Default)]
pub(crate) struct ShortcutManager {
    pub(crate) actions: gtk4::gio::SimpleActionGroup,
    pub(crate) shortcut_ctl: relm4::gtk::ShortcutController,
}

impl ShortcutManager {
    pub(crate) fn make<F: Fn(&gtk4::gio::SimpleAction, Option<&glib::Variant>) + 'static>(
        &self,
        shortcut: &'static str,
        name: &'static str,
        f: F,
    ) {
        let action = gtk4::gio::SimpleAction::new(name, None);
        action.connect_activate(f);
        self.actions.add_action(&action);
        let kb_shortcut = gtk4::Shortcut::builder()
            .trigger(&gtk4::ShortcutTrigger::parse_string(shortcut).unwrap())
            .action(&gtk4::ShortcutAction::parse_string(&format!("action(app.{name})")).unwrap())
            .build();
        self.shortcut_ctl.add_shortcut(kb_shortcut);
    }
}

#[macro_export]
macro_rules! gen_shortcut {
    ($shortcutman:ident $sender:ident) => {
        macro_rules! shortcut {
            ($shortcut:literal => $name:ident) => { ::paste::paste !{
                let new_sender = $sender.clone();
                $shortcutman.make($shortcut, stringify!([<$name:lower>]), move |_, _| new_sender.input(AppMsg::[<$name:camel>]));
            }};
            ($shortcut:literal => $name:ident => $alias:ident) => { ::paste::paste !{
                let new_sender = $sender.clone();
                $shortcutman.make($shortcut, stringify!([<$name:lower>]), move |_, _| new_sender.input(AppMsg::$alias));
            }};
            ($shortcut:literal => $name:ident => $body:block) => { ::paste::paste !{{
                #[allow(unused_variables)]
                let [<sender>] = $sender.clone();
                $shortcutman.make($shortcut, stringify!([<$name:lower>]), move |_, _| $body);
            }}};
        }
    }
}
