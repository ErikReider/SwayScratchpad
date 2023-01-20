mod application;
mod constants;
mod list_item;
mod utils;
mod window;

#[macro_use]
extern crate shrinkwraprs;

#[macro_use]
extern crate cascade;

use application::ScratchpadApplication;

fn main() {
	if gtk::init().is_err() {
		eprintln!("failed to initialize GTK Application");
		std::process::exit(1);
	}
	std::process::exit(ScratchpadApplication::new().start());
}
