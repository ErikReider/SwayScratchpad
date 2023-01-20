use std::path::Path;

use gtk::gio::DesktopAppInfo;
use swayipc::{Connection, Node};

pub fn get_swayipc_conn() -> Connection {
	match Connection::new() {
		Ok(c) => c,
		Err(e) => {
			eprintln!("Error: {}", e);
			std::process::exit(1);
		}
	}
}

pub fn get_scratchpad_applications() -> Vec<Node> {
	let mut conn = get_swayipc_conn();
	// get info from focused node and parent node which unfortunately requires us to call get_tree
	let tree = conn.get_tree().map_err(|_| "get_tree() failed").unwrap();
	let scratchpad = tree
		.find_as_ref(|n| n.name == Some("__i3_scratch".to_owned()))
		.ok_or("Could not find the scratchpad node")
		.unwrap();

	return scratchpad.floating_nodes.clone();
}

pub fn show_scratchpad_application(node: &Node) {
	let mut conn = get_swayipc_conn();
	match conn.run_command(format!("[con_id={}] scratchpad show", node.id)) {
		Ok(_) => (),
		Err(e) => eprintln!("SwayIPC Error: {}", e),
	}
}

pub fn node_get_class(node: &Node) -> Option<String> {
	match node.window_properties.clone() {
		Some(wp) => wp.class,
		_ => None,
	}
}

pub fn node_get_instance(node: &Node) -> Option<String> {
	match node.window_properties.clone() {
		Some(wp) => wp.instance,
		_ => None,
	}
}

pub fn node_get_title(node: &Node) -> Option<String> {
	match node.window_properties.clone() {
		Some(w) if w.title.is_some() => w.title,
		_ => node.name.clone(),
	}
}

pub fn node_get_role(node: &Node) -> Option<String> {
	match node.window_properties.clone() {
		Some(w) => w.window_role,
		None => None,
	}
}

// TODO: Clean up this mess...
pub fn get_desktop_file_from_node(node: &Node) -> Option<DesktopAppInfo> {
	let path = Path::new("/proc")
		.join(node.pid.unwrap().to_string())
		.join("exe");
	if path.exists() && path.is_symlink() {
		let link = path.read_link().unwrap();
		let base_name = link.file_name().unwrap().to_str().unwrap();

		// Try multiple methods of getting .desktop file
		let name_tries: Vec<Option<String>> = vec![
			node.app_id.clone(),
			node_get_class(node),
			Some(base_name.to_owned()),
			node_get_instance(node),
		];

		// Lookup .desktop files directly by file name
		for try_name in &name_tries {
			if let Some(try_name) = try_name {
				match DesktopAppInfo::new(&format!("{}.desktop", try_name)) {
					Some(desktop_file) => return Some(desktop_file),
					None => (),
				}
			}
		}

		// Try searching for .desktop files by names and filter by roles
		let role = node_get_role(node).unwrap_or("".to_owned()).to_lowercase();
		for try_name in &name_tries {
			if let Some(try_name) = try_name {
				let results = DesktopAppInfo::search(&try_name);
				if results.len() == 0 {
					continue;
				}
				for entry in results[0].clone() {
					let desktop = DesktopAppInfo::new(&entry);
					if let Some(desktop) = desktop {
						if role.len() > 0 {
							let keywords = desktop.keywords().join(";");
							if !keywords.to_lowercase().contains(&role) {
								continue;
							}
						}
						return Some(desktop);
					}
				}
			}
		}
	}
	return None;
}
