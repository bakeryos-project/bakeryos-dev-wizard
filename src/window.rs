use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use adw::subclass::prelude::*;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::components::card::Card;
use crate::install_window::BakeryOSDevWizardInstallProgressWindow;
use crate::models::event::Event;
use crate::models::package_info::{PackageGroup, PackageInfo};
use crate::services::package_service::PackageService;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/org/bakeryos/devwizard/window.ui")]
    #[properties(wrapper_type = super::BakeryOSDevWizardWindow)]
    pub struct BakeryOSDevWizardWindow {
        #[property(get, set)]
        pub packages_selected: Rc<RefCell<Vec<String>>>,

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
    impl ObjectSubclass for BakeryOSDevWizardWindow {
        const NAME: &'static str = "BakeryOSDevWizardWindow";
        type Type = super::BakeryOSDevWizardWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BakeryOSDevWizardWindow {}
    impl WidgetImpl for BakeryOSDevWizardWindow {}
    impl WindowImpl for BakeryOSDevWizardWindow {}
    impl ApplicationWindowImpl for BakeryOSDevWizardWindow {}
    impl AdwApplicationWindowImpl for BakeryOSDevWizardWindow {}
}

glib::wrapper! {
    pub struct BakeryOSDevWizardWindow(ObjectSubclass<imp::BakeryOSDevWizardWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl BakeryOSDevWizardWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        let obj: BakeryOSDevWizardWindow = glib::Object::builder()
            .property("application", application)
            .build();

        let imp = obj.imp();
        imp.show_packages();

        obj
    }
}

impl imp::BakeryOSDevWizardWindow {
    fn show_packages(&self) {
        let (sender, receiver) = async_channel::unbounded::<Event>();
        let list = match PackageService::load_packages() {
            Ok(p) => p,
            Err(e) => {
                println!("{e}");
                Arc::new(vec![])
            }
        };
        for package in list.deref() {
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

        self.install_button.connect_clicked(clone!(
            #[strong(rename_to = window)]
            self.obj(),
            #[strong(rename_to = package_selected)]
            self.packages_selected,
            move |_button| {
                let selected_ids = package_selected.borrow();
                let selected_packages: Vec<PackageInfo> = list
                    .iter()
                    .filter(|pkg| selected_ids.contains(&pkg.id))
                    .cloned()
                    .collect();
                let progress_window =
                    BakeryOSDevWizardInstallProgressWindow::new(selected_packages);
                progress_window.set_transient_for(Some(&window));
                progress_window.present();

                let progress_win_clone = progress_window.clone();
                progress_window
                    .imp()
                    .cancel_button()
                    .connect_clicked(move |_| {
                        progress_win_clone.close();
                    });
            }
        ));

        glib::spawn_future_local(clone!(
            #[strong(rename_to = package_selected)]
            self.packages_selected,
            async move {
                while let Ok(event) = receiver.recv().await {
                    match event.name.as_str() {
                        "select-package" => {
                            let package_id = event.package_id;

                            let mut vec = package_selected.borrow_mut();
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
        ));
    }
}
