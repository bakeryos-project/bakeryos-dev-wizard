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
        imp.load_data();
        imp.show_packages(sender);

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

    pub async fn listen_event_from_receiver(&self, receiver: Receiver<Event>) {
        while let Ok(event) = receiver.recv().await {
            match event.name.as_str() {
                "select-package" => {
                    let package_id = event.package_id;

                    let mut vec = self.packages_selected.borrow_mut();
                    if let Some(pos) = vec.iter().position(|x| x == &package_id) {
                        vec.remove(pos);
                    } else {
                        vec.push(package_id);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn on_install_button_clicked(&self) {
        let selected_ids = self.packages_selected.borrow();
        let selected_packages = self
            .packages
            .borrow()
            .iter()
            .filter(|pkg| selected_ids.contains(&pkg.id))
            .cloned()
            .collect();

        let window = self.obj();
        let progress_window = InstallProgressWindow::new(selected_packages);
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
