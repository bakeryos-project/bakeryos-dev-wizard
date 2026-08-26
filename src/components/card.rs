use std::sync::Arc;

use adw::subclass::prelude::*;
use async_channel::Sender;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::models::event::Event;
use crate::models::package_info::PackageInfo;

mod imp {
    use std::cell::Cell;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/org/bakeryos/devwizard/components/card.ui")]
    #[properties(wrapper_type = super::Card)]
    pub struct Card {
        #[property(get, set)]
        pub is_added: Cell<bool>,

        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub desc_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub details_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub package_icon: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Card {
        const NAME: &'static str = "Card";
        type Type = super::Card;
        type ParentType = gtk::FlowBoxChild;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Card {}
    impl WidgetImpl for Card {}
    impl BoxImpl for Card {}
    impl FlowBoxChildImpl for Card {}
}

glib::wrapper! {
    pub struct Card(ObjectSubclass<imp::Card>)
        @extends gtk::Widget,gtk::FlowBoxChild,  @implements gio::ActionGroup, gio::ActionMap;
}

impl Card {
    pub fn new(package_info: Arc<PackageInfo>, tx: Sender<Event>) -> Self {
        let obj: Card = glib::Object::new();
        let imp = obj.imp();
        imp.is_added.set(false);
        imp.show(&package_info);

        let pkg_info_1 = package_info.clone();
        imp.details_button.connect_clicked(clone!(
            #[strong]
            pkg_info_1,
            move |_| {
                let _ = webbrowser::open(&pkg_info_1.url);
            }
        ));

        let pkg_id = package_info.id.clone();
        let tx_clone = tx.clone();
        let obj_for_add = obj.clone();

        imp.add_button.connect_clicked(move |_| {
            let imp_inner = obj_for_add.imp();
            imp_inner.on_add_btn_clicked(&tx_clone, &pkg_id);
        });
        obj
    }
}

impl imp::Card {
    pub fn show(&self, package: &PackageInfo) {
        self.is_added.set(false);
        self.title_label.set_label(&package.name);
        self.desc_label.set_label(&package.description);

        match package.icon.as_deref() {
            Some(icon) => {
                self.fetch_icon(icon);
            }
            None => {
                let image: &gtk::Image = &self.package_icon.as_ref();
                image.set_icon_name(Some("application-x-executable-symbolic"));
            }
        }
    }

    pub fn on_details_button_clicked(&self, package: &PackageInfo) {
        let _ = webbrowser::open(&package.url);
    }

    pub fn on_add_btn_clicked(&self, tx: &Sender<Event>, package_id: &str) {
        let tx = tx.clone();
        let current = self.is_added.get();
        self.is_added.set(!current);

        if !current {
            self.add_button.set_label("Unselect");
            self.add_button.remove_css_class("suggested-action");
        } else {
            self.add_button.set_label("Add");
            self.add_button.add_css_class("suggested-action");
        }

        let package_id = package_id.to_string();
        glib::spawn_future_local(clone!(async move {
            let _ = tx
                .send(Event {
                    name: "select-package".to_owned(),
                    package_id: package_id.clone().to_owned(),
                })
                .await;
        }));
    }

    pub fn fetch_icon(&self, icon_url: &str) {
        let icon_url = icon_url.to_owned();
        let app_icon_widget = self.package_icon.clone();

        glib::spawn_future_local(async move {
            let download_result = gio::spawn_blocking(move || {
                let response = ureq::get(&icon_url).call()?;

                let mut bytes_data = Vec::new();
                response.into_reader().read_to_end(&mut bytes_data)?;

                Ok::<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>(bytes_data)
            })
            .await;

            match download_result {
                Ok(Ok(bytes_data)) => {
                    let bytes = glib::Bytes::from(&bytes_data);
                    if let Ok(texture) = gtk::gdk::Texture::from_bytes(&bytes) {
                        let image: &gtk::Image = app_icon_widget.as_ref();
                        image.set_property("paintable", &texture);
                    }
                }
                _ => {
                    let image: &gtk::Image = app_icon_widget.as_ref();
                    image.set_icon_name(Some("application-x-executable-symbolic"));
                }
            }
        });
    }
}
