use std::cell::RefCell;
use std::rc::Rc;

use gtk::gio::{ApplicationFlags, Cancellable};
use gtk::glib::variant::DictEntry;
use gtk::glib::{OptionArg, OptionFlags, SignalHandlerId, Variant, VariantTy};
use gtk::prelude::*;
use gtk::*;

use crate::window::ScratchpadWindow;

use gtk::gio::SimpleAction;

const ACTION_NAME: &str = "action";
const ACTION_FORMAT: &str = "s";

#[derive(Debug, PartialEq)]
pub enum ActionTypes {
	None = 0,
	Show = 1,
	Hide = 2,
	Toggle = 3,
}
impl ActionTypes {
	pub fn as_str(&self) -> &'static str {
		match self {
			ActionTypes::None => "NONE",
			ActionTypes::Show => "SHOW",
			ActionTypes::Hide => "HIDE",
			ActionTypes::Toggle => "TOGGLE",
		}
	}

	pub fn parse(value: &str) -> Self {
		match value {
			"SHOW" => ActionTypes::Show,
			"HIDE" => ActionTypes::Hide,
			"TOGGLE" => ActionTypes::Toggle,
			_ => ActionTypes::None,
		}
	}
}

#[derive(Clone, Shrinkwrap)]
pub struct ScratchpadApplication {
	#[shrinkwrap(main_field)]
	app: gtk::Application,
	started: Rc<RefCell<bool>>,
	action_id: Rc<RefCell<Option<SignalHandlerId>>>,
	window: Rc<RefCell<Option<ScratchpadWindow>>>,
}

impl ScratchpadApplication {
	pub fn new() -> Self {
		let app = Application::new(
			Some("org.erikreider.swayscratchpad"),
			ApplicationFlags::FLAGS_NONE,
		);

		// Show Window cmdline arg
		app.add_main_option(
			"show",
			glib::Char::from(0),
			OptionFlags::NONE,
			OptionArg::None,
			"",
			None,
		);
		// Hide Window cmdline arg
		app.add_main_option(
			"hide",
			glib::Char::from(0),
			OptionFlags::NONE,
			OptionArg::None,
			"",
			None,
		);
		// Toggle Window cmdline arg
		app.add_main_option(
			"toggle",
			glib::Char::from(0),
			OptionFlags::NONE,
			OptionArg::None,
			"",
			None,
		);

		// Parse args
		app.connect_handle_local_options(|app, args| -> i32 {
			let variant = args.to_variant();
			if variant.n_children() > 1 {
				eprintln!("Only run with one arg at once!...");
				return 1;
			} else if variant.n_children() == 0 {
				return -1;
			}

			if !variant.is_container() {
				eprintln!("VariantDict isn't a container!...");
				return 1;
			}
			let child: DictEntry<String, Variant> = variant.child_get(0);
			let action_type: ActionTypes = match child.key().as_str() {
				"show" => ActionTypes::Show,
				"hide" => ActionTypes::Hide,
				"toggle" => ActionTypes::Toggle,
				e => {
					eprintln!("Unknown Variant Key: \"{}\"!...", e);
					return 1;
				}
			};
			app.activate_action(ACTION_NAME, Some(&action_type.as_str().to_variant()));
			return 0;
		});

		ScratchpadApplication {
			app,
			started: Rc::new(RefCell::new(false)),
			action_id: Rc::new(RefCell::new(None)),
			window: Rc::new(RefCell::new(None)),
		}
	}

	pub fn start(&self) -> i32 {
		let s = self.clone();
		self.app.connect_activate(move |_| {
			if s.started.borrow().to_owned() == true {
				return;
			}
			s.started.replace(true);
			s.window.replace(Some(ScratchpadWindow::new(&s.app)));
		});

		match VariantTy::new(ACTION_FORMAT) {
			Ok(variant_ty) => {
				let action = SimpleAction::new(ACTION_NAME, Some(variant_ty));
				let s = self.clone();
				self.action_id.replace(Some(
					action.connect_activate(move |sa, v| s.action_activated(sa, v)),
				));
				self.app.add_action(&action);
				let _ = self.app.register(Cancellable::NONE);
			}
			Err(x) => {
				eprintln!("VARIANT TYPE ERROR: {}", x.message);
				std::process::exit(1);
			}
		}

		return self.app.run();
	}

	fn action_activated(&self, action: &SimpleAction, variant: Option<&Variant>) {
		if self.started.borrow().to_owned() != true {
			eprintln!("Please start the executable separately before running with args!...");
			return;
		}
		match self.action_id.take() {
			Some(action_id) => action.disconnect(action_id),
			None => return,
		}

		if let Some(variant) = variant {
			match ActionTypes::parse(&variant.str().unwrap_or("")) {
				ActionTypes::None => {
					eprintln!("Failed to parse variant: {}!...", variant.print(true))
				}
				ActionTypes::Show => self.window.borrow().as_ref().unwrap().show(),
				ActionTypes::Hide => self.window.borrow().as_ref().unwrap().hide(),
				ActionTypes::Toggle => self.window.borrow().as_ref().unwrap().toggle(),
			};
		}

		let s = self.clone();
		let id = action.connect_activate(move |sa, v| s.action_activated(sa, v));
		self.action_id.replace(Some(id));
	}
}
