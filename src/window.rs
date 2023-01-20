use std::{cell::RefCell, rc::Rc, usize};

use gtk::{
	gdk::{
		keys::constants::{Caps_Lock, Escape},
		EventKey,
	},
	glib::clone,
	prelude::*,
	Adjustment, ApplicationWindow, PolicyType, SelectionMode,
};
use swayipc::Node;

use crate::{
	constants::{WINDOW_HEIGHT, WINDOW_PADDING, WINDOW_WIDTH},
	list_item::ListItem,
	utils::{get_scratchpad_applications, show_scratchpad_application},
};

/// A window that our application can open that contains the main project view.
#[derive(Clone, Debug)]
pub struct ScratchpadWindow {
	pub window: gtk::ApplicationWindow,
	list_items: Rc<RefCell<Vec<ListItem>>>,
	list_box: gtk::ListBox,
}

impl ScratchpadWindow {
	/// Create a new window and assign it to the given application.
	pub fn new(app: &gtk::Application) -> Self {
		let window = gtk::ApplicationWindow::new(app);
		window.set_title("Sway Scratchpad");
		window.set_default_size(WINDOW_WIDTH, -1);
		window.set_size_request(WINDOW_WIDTH, -1);

		gtk_layer_shell::init_for_window(&window);
		gtk_layer_shell::set_namespace(&window, "erikreider.swayscratchpad");
		gtk_layer_shell::auto_exclusive_zone_enable(&window);
		gtk_layer_shell::set_keyboard_interactivity(&window, true);

		gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);

		let container = cascade! {
			gtk::Box::new(gtk::Orientation::Vertical, 12);
			..set_margin(WINDOW_PADDING);
			..show();
		};
		window.add(&container);

		// List box title
		let title = cascade! {
			gtk::Label::new(Some("Select window to show"));
			..style_context().add_class("h2");
			..show();
		};
		container.add(&title);

		// ScrolledWindow
		let sw = cascade! {
			gtk::ScrolledWindow::new(Adjustment::NONE, Adjustment::NONE);
			..set_hscrollbar_policy(PolicyType::Never);
			..set_vscrollbar_policy(PolicyType::Automatic);
			..set_overlay_scrolling(false);
			..set_min_content_height(WINDOW_HEIGHT);
			..set_max_content_height(WINDOW_WIDTH);
			..set_propagate_natural_height(true);
			..show();
		};

		container.add(&sw);

		// Viewport
		let vp = cascade! {
			gtk::Viewport::new(Adjustment::NONE, Adjustment::NONE);
			..show();
		};
		sw.add(&vp);

		// Set up the list box
		let list_box = cascade! {
			gtk::ListBox::new();
			..style_context().add_class("content");
			..set_activate_on_single_click(true);
			..set_selection_mode(SelectionMode::Single);
			..show();
		};

		vp.add(&list_box);

		let scratchpad_win = Self {
			window,
			list_box,
			list_items: Rc::new(RefCell::new(vec![])),
		};

		scratchpad_win.window.connect_key_release_event(
			clone!(@strong scratchpad_win => move |a, e| scratchpad_win.clone().key_click_cb(a, e)),
		);

		scratchpad_win.list_box.connect_row_activated(
			clone!(@strong scratchpad_win as self_ => move |_, row| {
				if self_.list_items.as_ref().borrow().len() == 0 {
					return self_.hide();
				}

				let node: Node = {
					match self_.list_items.as_ref().borrow().get(row.index() as usize) {
						Some(item) => {
							assert!(item.widget.eq(row), "Item index is not synced!...");
							item.node.clone()
						},
						None => return eprintln!("Item index is not correct..."),
					}
				};
				show_scratchpad_application(&node);
				self_.hide();
			}),
		);

		return scratchpad_win;
	}

	/// Shows the window.
	/// Clears the old items and adds the new Scratchpad windows.
	pub fn show(&self) {
		if self.window.is_visible() {
			return;
		}
		self.add_items(get_scratchpad_applications());

		self.list_box.set_has_focus(true);

		self.window.set_visible(true);
	}

	/// Hides the window
	pub fn hide(&self) {
		self.window.set_visible(false);
		// Clear all container items
		for widget in self.list_box.children() {
			self.list_box.remove(&widget);
		}
		self.list_items.borrow_mut().clear();
	}

	/// Toggle the window
	pub fn toggle(&self) {
		if self.window.is_visible() {
			self.hide()
		} else {
			self.show()
		}
	}

	/// Add all node items
	fn add_items(&self, nodes: Vec<Node>) {
		if nodes.len() == 0 {
			// Add placeholder
			let placeholder = cascade! {
				gtk::ListBoxRow::new();
				..set_size_request(WINDOW_WIDTH, WINDOW_HEIGHT);
				..add(&cascade! {
					gtk::Box::new(gtk::Orientation::Vertical, 12);
					..set_valign(gtk::Align::Center);
					..set_opacity(0.5);
					..add(&cascade! {
						gtk::Image::new();
						..set_pixel_size(96);
						..set_from_icon_name(Some("application-x-executable-symbolic"), gtk::IconSize::Invalid);
					});
					..add(&gtk::Label::new(Some("No windows in scratchpad...")));
				});
				..show_all();
			};
			self.list_box.add(&placeholder);
			return;
		}

		for node in nodes {
			let list_item = ListItem::new(node);
			self.list_box.add(&list_item.widget);
			list_item.widget.queue_resize();
			self.list_items.borrow_mut().push(list_item);
		}
	}

	#[allow(non_upper_case_globals)]
	fn key_click_cb(&self, _app: &ApplicationWindow, event: &EventKey) -> gtk::Inhibit {
		match event.keyval() {
			Escape | Caps_Lock => self.hide(),
			_ => (),
		}
		return Inhibit(false);
	}
}
