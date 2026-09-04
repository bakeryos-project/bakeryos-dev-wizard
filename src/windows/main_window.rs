use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use adw::subclass::prelude::*;
use async_channel::{Receiver, Sender};
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::components::card::Card;
use crate::models::event::Event;
use crate::models::package_info::{PackageGroup, PackageInfo};
use crate::services::package_service::PackageService;
use crate::windows::install_window::InstallProgressWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/org/bakeryos/devwizard/window.ui")]
    #[properties(wrapper_type = super::MainWindow)]
    pub struct MainWindow {
        #[property(get, set)]
        pub packages_selected: Rc<RefCell<Vec<String>>>,

        pub packages: RefCell<Vec<PackageInfo>>,

        pub category_cards: RefCell<Vec<Card>>,
        pub search_cards: RefCell<Vec<Card>>,
        pub event_sender: RefCell<Option<Sender<Event>>>,

        #[template_child]
        pub search_button: TemplateChild<gtk::ToggleButton>,

        #[template_child]
        pub search_bar: TemplateChild<gtk::SearchBar>,

        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,

        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub search_result_stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub search_packages: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub search_empty_page: TemplateChild<adw::StatusPage>,

        #[template_child]
        pub search_summary_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub code_editor_packages: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub programming_language_packages: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub framework_packages: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub tooling_packages: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub ai_agent_packages: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub browser_packages: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub other_packages: TemplateChild<gtk::FlowBox>,

        #[template_child]
        pub install_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MainWindow {
        const NAME: &'static str = "MainWindow";
        type Type = super::MainWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MainWindow {}
    impl WidgetImpl for MainWindow {}
    impl WindowImpl for MainWindow {}
    impl ApplicationWindowImpl for MainWindow {}
    impl AdwApplicationWindowImpl for MainWindow {}
}

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<imp::MainWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl MainWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        let obj: MainWindow = glib::Object::builder()
            .property("application", application)
            .build();

        let (sender, receiver) = async_channel::unbounded::<Event>();
        let imp = obj.imp();
        imp.event_sender.replace(Some(sender.clone()));
        imp.load_data();
        imp.show_packages(sender.clone());

        // Connect search bar and key capture
        imp.search_bar.set_key_capture_widget(Some(&obj));
        imp.search_bar.connect_entry(&imp.search_entry.get());

        // Connect search entry changes
        imp.search_entry.connect_search_changed(clone!(
            #[weak]
            imp,
            #[strong]
            sender,
            move |entry| {
                imp.on_search_changed(entry.text().as_str(), &sender);
            }
        ));

        // When search bar is closed / toggled off, reset search
        imp.search_bar.connect_search_mode_enabled_notify(clone!(
            #[weak]
            imp,
            move |bar| {
                if !bar.is_search_mode() {
                    imp.search_entry.set_text("");
                    imp.content_stack.set_visible_child_name("categories");
                    imp.clear_search_results();
                    imp.search_summary_label.set_label("");
                }
            }
        ));

        // Setup Ctrl+F shortcut
        let shortcut = gtk::Shortcut::builder()
            .trigger(&gtk::ShortcutTrigger::parse_string("<Control>f").unwrap())
            .action(&gtk::NamedAction::new("win.search"))
            .build();
        let controller = gtk::ShortcutController::new();
        controller.add_shortcut(shortcut);
        obj.add_controller(controller);

        let search_action = gio::SimpleAction::new("search", None);
        search_action.connect_activate(clone!(
            #[weak]
            imp,
            move |_, _| {
                let active = imp.search_button.is_active();
                imp.search_button.set_active(!active);
                if !active {
                    imp.search_entry.grab_focus();
                }
            }
        ));
        obj.add_action(&search_action);

        imp.install_button.connect_clicked(clone!(
            #[weak]
            imp,
            move |_button| {
                imp.on_install_button_clicked();
            }
        ));

        glib::spawn_future_local(clone!(
            #[weak]
            imp,
            async move {
                imp.listen_event_from_receiver(receiver).await;
            }
        ));

        obj
    }
}

impl imp::MainWindow {
    fn load_data(&self) {
        let mut packages = self.packages.borrow_mut();
        *packages = match PackageService::load_packages() {
            Ok(p) => p,
            Err(e) => {
                println!("{e}");
                vec![]
            }
        };
    }

    fn show_packages(&self, sender: Sender<Event>) {
        for package in self.packages.borrow().iter() {
            let group: PackageGroup = package.group.clone();
            let p = Arc::new(package.to_owned());
            let card = Card::new(p, sender.clone());
            self.category_cards.borrow_mut().push(card.clone());
            match group {
                PackageGroup::CodeEditor => {
                    self.code_editor_packages.append(&card);
                }

                PackageGroup::ProgrammingLanguage => {
                    self.programming_language_packages.append(&card);
                }

                PackageGroup::Framework => {
                    self.framework_packages.append(&card);
                }

                PackageGroup::Tooling => {
                    self.tooling_packages.append(&card);
                }

                PackageGroup::AIAgent => {
                    self.ai_agent_packages.append(&card);
                }

                PackageGroup::Browser => {
                    self.browser_packages.append(&card);
                }

                PackageGroup::Other => {
                    self.other_packages.append(&card);
                }
            }
        }
    }

    fn on_search_changed(&self, query: &str, sender: &Sender<Event>) {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            self.content_stack.set_visible_child_name("categories");
            self.clear_search_results();
            self.search_summary_label.set_label("");
        } else {
            self.content_stack.set_visible_child_name("search");
            self.perform_search(trimmed, sender);
        }
    }

    fn clear_search_results(&self) {
        while let Some(child) = self.search_packages.first_child() {
            self.search_packages.remove(&child);
        }
        self.search_cards.borrow_mut().clear();
    }

    fn perform_search(&self, query: &str, sender: &Sender<Event>) {
        self.clear_search_results();

        let query_lower = query.to_lowercase();
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();

        let mut matched: Vec<(usize, PackageInfo)> = Vec::new();

        for pkg in self.packages.borrow().iter() {
            let name_lower = pkg.name.to_lowercase();
            let id_lower = pkg.id.to_lowercase();
            let desc_lower = pkg.description.to_lowercase();

            let matches_all = keywords.iter().all(|kw| {
                name_lower.contains(kw) || id_lower.contains(kw) || desc_lower.contains(kw)
            });

            if matches_all {
                let score = if name_lower == query_lower || id_lower == query_lower {
                    0
                } else if name_lower.starts_with(&query_lower) || id_lower.starts_with(&query_lower)
                {
                    1
                } else if name_lower.contains(&query_lower) || id_lower.contains(&query_lower) {
                    2
                } else {
                    3
                };
                matched.push((score, pkg.clone()));
            }
        }

        matched.sort_by(|a, b| {
            a.0.cmp(&b.0).then_with(|| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()))
        });

        let count = matched.len();
        if count == 0 {
            self.search_summary_label.set_label("");
            self.search_empty_page
                .set_title(&format!("No packages found for \"{}\"", query));
            self.search_empty_page
                .set_description(Some("Try searching with different keywords"));
            self.search_result_stack.set_visible_child_name("empty");
        } else {
            let summary_text = if count == 1 {
                "1 package found".to_string()
            } else {
                format!("{} packages found", count)
            };
            self.search_summary_label.set_label(&summary_text);
            self.search_result_stack.set_visible_child_name("results");

            let selected_ids = self.packages_selected.borrow();
            for (_, pkg) in matched {
                let is_selected = selected_ids.contains(&pkg.id);
                let card = Card::new(Arc::new(pkg), sender.clone());
                card.set_selected(is_selected);
                self.search_packages.append(&card);
                self.search_cards.borrow_mut().push(card);
            }
        }
    }

    pub async fn listen_event_from_receiver(&self, receiver: Receiver<Event>) {
        while let Ok(event) = receiver.recv().await {
            match event.name.as_str() {
                "select-package" => {
                    let package_id = event.package_id;

                    let is_selected = {
                        let mut vec = self.packages_selected.borrow_mut();
                        if let Some(pos) = vec.iter().position(|x| x == &package_id) {
                            vec.remove(pos);
                            false
                        } else {
                            vec.push(package_id.clone());
                            true
                        }
                    };

                    self.update_card_state(&package_id, is_selected);
                }
                "unselect-package" => {
                    let package_id = event.package_id;
                    let mut vec = self.packages_selected.borrow_mut();
                    if let Some(pos) = vec.iter().position(|x| x == &package_id) {
                        vec.remove(pos);
                    }
                    self.update_card_state(&package_id, false);
                }
                "install-completed" => {
                    self.clear_selected_packages();
                }
                _ => {}
            }
        }
    }

    fn clear_selected_packages(&self) {
        let ids: Vec<String> = self.packages_selected.borrow().clone();
        for id in &ids {
            self.update_card_state(id, false);
        }
        self.packages_selected.borrow_mut().clear();
    }

    fn update_card_state(&self, package_id: &str, is_selected: bool) {
        for card in self.category_cards.borrow().iter() {
            if card.package_id() == package_id {
                card.set_selected(is_selected);
            }
        }
        for card in self.search_cards.borrow().iter() {
            if card.package_id() == package_id {
                card.set_selected(is_selected);
            }
        }
    }

    pub fn on_install_button_clicked(&self) {
        let selected_ids = self.packages_selected.borrow();
        let selected_packages: Vec<PackageInfo> = self
            .packages
            .borrow()
            .iter()
            .filter(|pkg| selected_ids.contains(&pkg.id))
            .cloned()
            .collect();

        if selected_packages.is_empty() {
            return;
        }

        let sender = match &*self.event_sender.borrow() {
            Some(s) => s.clone(),
            None => return,
        };

        let window = self.obj();
        let progress_window = InstallProgressWindow::new(selected_packages, sender);
        progress_window.set_transient_for(Some(window.as_ref()));
        progress_window.present();

        let progress_win_clone = progress_window.clone();
        progress_window
            .imp()
            .cancel_button()
            .connect_clicked(move |_| {
                progress_win_clone.close();
            });
    }
}
