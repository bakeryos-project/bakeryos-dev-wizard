use adw::prelude::{ActionRowExt, PreferencesRowExt};
use adw::subclass::prelude::*;
use async_channel::Sender;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::models::event::Event;
use crate::models::package_info::PackageInfo;
use crate::services::package_service::PackageService;

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bakeryos/devwizard/install_window.ui")]

    pub struct InstallProgressWindow {
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub header_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub package_list_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InstallProgressWindow {
        const NAME: &'static str = "InstallProgressWindow";
        type Type = super::InstallProgressWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for InstallProgressWindow {}
    impl WidgetImpl for InstallProgressWindow {}
    impl WindowImpl for InstallProgressWindow {}
    impl AdwWindowImpl for InstallProgressWindow {}
}

glib::wrapper! {
    pub struct InstallProgressWindow(ObjectSubclass<imp::InstallProgressWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::Window,        @implements gio::ActionGroup, gio::ActionMap;
}

impl InstallProgressWindow {
    pub fn new(packages: Vec<PackageInfo>, sender: Sender<Event>) -> Self {
        let obj: Self = glib::Object::new();

        glib::spawn_future_local(clone!(
            #[strong]
            obj,
            async move {
                obj.imp().start_install(packages, sender).await;
            }
        ));

        obj
    }
}

impl imp::InstallProgressWindow {
    pub async fn start_install(&self, packages: Vec<PackageInfo>, sender: Sender<Event>) {
        match PackageService::authenticate() {
            Ok(_) => {}
            Err(e) => {
                self.show_err_status(&e);
                return;
            }
        };

        let total_packages = packages.len() as f64;
        for (i, package) in packages.into_iter().enumerate() {
            let fraction = (i + 1) as f64 / total_packages;
            let (row, suffix_box) = self.build_list_package_row(&package.name);
            self.package_list_box.append(&row);
            self.set_progress(fraction);

            let result = self.spawn_install_task(&package).await;
            while let Some(child) = suffix_box.first_child() {
                suffix_box.remove(&child);
            }

            match result {
                Ok(_) => {
                    row.set_subtitle("Installed");
                    row.add_prefix(&gtk::Image::from_icon_name("object-select-symbolic"));
                    let _ = sender
                        .send(Event {
                            name: "unselect-package".to_owned(),
                            package_id: package.id.clone(),
                        })
                        .await;
                }
                Err(e) => {
                    eprintln!("Failed {}: {}", package.name, e);
                    row.set_subtitle(&format!("Error: {}", e));
                    row.add_prefix(&gtk::Image::from_icon_name("dialog-error-symbolic"));
                    row.add_css_class("error");
                }
            }
        }

        self.show_done_status();
        let _ = sender
            .send(Event {
                name: "install-completed".to_owned(),
                package_id: String::new(),
            })
            .await;
    }

    pub fn show_err_status(&self, err: &str) {
        self.set_progress(100.0);
        self.header_label.set_label("Error");
        self.status_label.set_label(err);

        let cancel_btn = self.cancel_button();
        cancel_btn.set_label("Close");
        cancel_btn.remove_css_class("destructive-action");
    }

    pub fn show_done_status(&self) {
        self.set_progress(100.0);
        self.header_label.set_label("Done");
        self.status_label.set_label("");
        let cancel_btn = self.cancel_button();
        cancel_btn.set_label("Done");
        cancel_btn.remove_css_class("destructive-action");
    }

    pub async fn spawn_install_task(&self, package: &PackageInfo) -> Result<(), String> {
        let package = package.clone();
        let result = gio::spawn_blocking(move || PackageService::install_pkg(&package)).await;
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Install task error".to_owned()),
        }
    }

    pub fn build_list_package_row(&self, pkg_name: &str) -> (adw::ActionRow, gtk::Box) {
        let row = adw::ActionRow::new();
        row.set_title(&pkg_name);
        row.set_subtitle("Installing ...");

        let suffix_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let spinner = adw::Spinner::new();
        suffix_box.append(&spinner);
        row.add_suffix(&suffix_box);

        (row, suffix_box)
    }

    pub fn set_progress(&self, fraction: f64) {
        self.progress_bar.set_fraction(fraction);
    }

    pub fn cancel_button(&self) -> gtk::Button {
        self.cancel_button.clone()
    }
}
