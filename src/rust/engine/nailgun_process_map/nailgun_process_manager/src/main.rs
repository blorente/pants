
use sysinfo::{SystemExt};
use std::collections::btree_map::BTreeMap;

use process_execution::{ExecuteProcessRequest, Platform};
use nailgun_process_map::NailgunProcessMap;
use std::time::Duration;
use std::collections::btree_set::BTreeSet;

fn main() {
    println!("Hello, world!");
    let tool_name = String::from("zinc");
    let system = sysinfo::System::new();
    let startup_options = ExecuteProcessRequest {
        argv: vec![String::from("watch"), String::from("ls")],
//        argv: vec![String::from("/Users/bescobar/workspace/otherpants/src/rust/engine/target/debug/nailgun_process_manager")],
        env: BTreeMap::new(),
        input_files: Default::default(),
        output_files: BTreeSet::new(),
        output_directories: BTreeSet::new(),
        timeout: Duration::new(1000, 0),
        description: String::from("EPR to start a nailgun"),
        jdk_home: None,
        target_platform: Platform::Darwin,
    };
    let mut map = NailgunProcessMap::new();
    let process = map.connect(tool_name, startup_options);
    println!("Got process! {:?}", process.expect("TODO"))
}
