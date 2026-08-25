use adw::subclass::prelude::*;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::models::package_info::PackageInfo;
use crate::services::package_service::PackageService;

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/org/bakeryos/devwizard/install_window.ui")]

    pub struct BakeryOSDevWizardInstallProgressWindow {
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub header_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BakeryOSDevWizardInstallProgressWindow {
        const NAME: &'static str = "BakeryOSDevWizardInstallProgressWindow";
        type Type = super::BakeryOSDevWizardInstallProgressWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BakeryOSDevWizardInstallProgressWindow {}
    impl WidgetImpl for BakeryOSDevWizardInstallProgressWindow {}
    impl WindowImpl for BakeryOSDevWizardInstallProgressWindow {}
    impl AdwWindowImpl for BakeryOSDevWizardInstallProgressWindow {}
}

glib::wrapper! {
    pub struct BakeryOSDevWizardInstallProgressWindow(ObjectSubclass<imp::BakeryOSDevWizardInstallProgressWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::Window,        @implements gio::ActionGroup, gio::ActionMap;
}

impl BakeryOSDevWizardInstallProgressWindow {
    pub fn new(packages: Vec<PackageInfo>) -> Self {
        let obj: Self = glib::Object::new();
        let total_packages = packages.len() as f64;

        glib::spawn_future_local(clone!(
            #[weak(rename_to=window)]
            obj,
            #[weak(rename_to=imp)]
            obj.imp(),
            async move {
                PackageService::authenticate();
                let mut error_count: f64 = 0.0;
                for (i, package) in packages.into_iter().enumerate() {
                    let current_num = i + 1;
                    let fraction = current_num as f64 / total_packages;
                    let pkg_name = package.name.clone();

                    imp.set_progress(
                        fraction,
                        &format!(
                            "({}/{}) Installing {}",
                            current_num, total_packages, pkg_name
                        ),
                    );

                    let pkg_clone = package.clone();
                    let result =
                        gio::spawn_blocking(move || PackageService::install_pkg(&pkg_clone)).await;

                    match result {
                        Ok(Ok(())) => {
                            println!("Installed: {}", pkg_name);
                        }
                        Ok(Err(e)) => {
                            eprintln!("Failed {}: {}", pkg_name, e);
                            error_count += 1.0;
                        }
                        Err(_) => {
                            error_count += 1.0;
                        }
                    }
                }
                imp.set_progress(
                    100.0,
                    &format!(
                        "Success: {} | Error: {}",
                        total_packages - error_count,
                        error_count
                    ),
                );

                let cancel_btn = imp.cancel_button();
                imp.header_label.get().set_label("Done");
                cancel_btn.set_label("Done");
                cancel_btn.remove_css_class("destructive-action");
            }
        ));
        obj
    }
}

impl imp::BakeryOSDevWizardInstallProgressWindow {
    pub fn set_progress(&self, fraction: f64, text: &str) {
        self.progress_bar.set_fraction(fraction);
        self.status_label.set_text(text);
    }

    pub fn cancel_button(&self) -> gtk::Button {
        self.cancel_button.clone()
    }
}
