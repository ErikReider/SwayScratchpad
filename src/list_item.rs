use gtk::prelude::*;
use swayipc::Node;

use crate::{
	constants::{LIST_ITEM_IMAGE_SIZE, LIST_ITEM_MARGIN},
	utils::{get_desktop_file_from_node, node_get_title},
};

/// A window that our application can open that contains the main project view.
#[derive(Clone, Debug)]
pub struct ListItem {
	pub widget: gtk::ListBoxRow,
	pub node: Node,
}

impl ListItem {
	/// Create a new window and assign it to the given application.
	pub fn new(node: Node) -> Self {
		let widget = cascade! {
			gtk::ListBoxRow::new();
			..show();
		};
		widget.set_activatable(true);
		widget.set_selectable(false);

		// Set up a widget
		let container = cascade! {
			gtk::Box::new(gtk::Orientation::Horizontal, 12);
			..set_margin(LIST_ITEM_MARGIN);
			..show();
		};
		widget.add(&container);

		let image = cascade! {
			gtk::Image::new();
			..show();
			..set_pixel_size(LIST_ITEM_IMAGE_SIZE);
			..set_size_request(LIST_ITEM_IMAGE_SIZE, LIST_ITEM_IMAGE_SIZE);
		};
		container.add(&image);

		let mut name = "".to_owned();
		Self::set_icon_from_node(&node, &image, &mut name);
		let mut label_str = Self::get_label_from_node(&node);
		if name.trim().len() > 0 && name.to_lowercase() != label_str.to_lowercase() {
			label_str = format!("{} - {}", name, label_str);
		}

		let label = cascade! {
			gtk::Label::new(Some(&label_str));
			..set_ellipsize(gtk::pango::EllipsizeMode::End);
			..set_justify(gtk::Justification::Left);
			..set_wrap(true);
			..set_line_wrap(true);
			..set_lines(3);
			..set_wrap_mode(gtk::pango::WrapMode::WordChar);
			..show();
		};
		container.add(&label);

		Self { widget, node }
	}

	fn set_icon_from_node(node: &Node, image: &gtk::Image, name: &mut String) {
		if let Some(desktop_file) = get_desktop_file_from_node(node) {
			*name = desktop_file.display_name().to_string();
			if let Some(icon) = desktop_file.icon() {
				image.set_from_gicon(&icon, gtk::IconSize::Invalid);
				return;
			}
		}
		image.set_from_icon_name(Some("image-missing"), gtk::IconSize::Invalid);
	}

	fn get_label_from_node(node: &Node) -> String {
		node_get_title(node).unwrap_or(format!("Window ID: {}", node.id))
	}
}
