
use sysinfo::{SystemExt};
use std::collections::btree_map::BTreeMap;

use process_execution::{ExecuteProcessRequest, Platform};
use nailgun_process_map::NailgunProcessMap;
use std::time::Duration;
use std::collections::btree_set::BTreeSet;
use std::io::Read;

fn main() {
    let tool_name = String::from("zinc");
    let startup_options = ExecuteProcessRequest {
        argv: vec![
            String::from("/Library/Java/JavaVirtualMachines/TwitterJDK/Contents/Home/bin/java"),
            String::from("-Xmx1g"),
            String::from("-Dpants.buildroot=/Users/bescobar/workspace/otherpants"),
            String::from("-Dpants.nailgun.owner=/Users/bescobar/workspace/otherpants/.pants.d/ng/FindBugs_compile_findbugs"),
            String::from("-Dpants.nailgun.fingerprint=a89c6538bef5aabf182b06a81f910a66c87d28eb"),
            String::from("-cp"),
            String::from("/Users/bescobar/workspace/otherpants/.pants.d/bootstrap/bootstrap-jvm-tools/a0ebe8e0b001/ivy/jars/com.martiansoftware/nailgun-server/jars/nailgun-server-0.9.1.jar:/Users/bescobar/workspace/otherpants/.pants.d/bootstrap/bootstrap-jvm-tools/tool_cache/shaded_jars/edu.umd.cs.findbugs.FindBugs2-d67ca5946d9e39768284080cf50baef9e5a05412-ShadedToolFingerprintStrategy_c6111d2fe766.jar"),
            String::from("com.martiansoftware.nailgun.NGServer"),
            String::from(":0")
        ],
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

    loop {
            map.connect(tool_name.clone(), startup_options.clone()).map(|_| {
                let process = map.get(&tool_name);
                println!("Got process! {:#?}", process.expect("TODO"));
            });
            pause();
    }
}

fn pause() {
    use std::io::prelude::*;
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}