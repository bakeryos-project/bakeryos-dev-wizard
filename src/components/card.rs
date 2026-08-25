/* card.rs
 *
 * Copyright 2026 smtdfc
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::sync::Arc;

use adw::subclass::prelude::*;
use async_channel::Sender;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::models::event::Event;
use crate::models::package_info::{self, PackageInfo};

mod imp {
    use std::{cell::Cell, sync::Arc};

    use gtk::FlowBox;

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
        imp.title_label.set_label(&package_info.name);
        imp.desc_label.set_label(&package_info.description);

        match package_info.icon.as_deref() {
            Some(icon) => {
                imp.fetch_icon(icon);
            }
            None => {
                let image: &gtk::Image = &imp.package_icon.as_ref();
                image.set_icon_name(Some("application-x-executable-symbolic"));
            }
        }
        let pkg = package_info.clone();
        imp.details_button.connect_clicked(move |_| {
            let url = pkg.url.clone();
            let _ = webbrowser::open(&url);
        });

        let pkg = package_info.clone();
        imp.add_button.connect_clicked(clone!(
            #[weak]
            pkg,
            #[weak]
            imp,
            move |button| {
                let tx = tx.clone();
                let current = imp.is_added.get();
                imp.is_added.set(!current);

                if !current {
                    button.set_label("Unselect");
                    button.remove_css_class("suggested-action");
                } else {
                    button.set_label("Add");
                    button.add_css_class("suggested-action");
                }

                glib::spawn_future_local(async move {
                    let _ = tx
                        .send(Event {
                            name: "select-package".to_owned(),
                            package_id: pkg.id.clone(),
                        })
                        .await;
                });
            }
        ));
        obj
    }
}

impl imp::Card {
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
