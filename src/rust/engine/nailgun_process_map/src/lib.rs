// Copyright 2019 Pants project contributors (see CONTRIBUTORS.md).
// Licensed under the Apache License, Version 2.0 (see LICENSE).

//#![deny(warnings)]
// Enable all clippy lints except for many of the pedantic ones. It's a shame this needs to be copied and pasted across crates, but there doesn't appear to be a way to include inner attributes from a common source.
#![deny(
clippy::all,
clippy::default_trait_access,
clippy::expl_impl_clone_on_copy,
clippy::if_not_else,
clippy::needless_continue,
clippy::single_match_else,
clippy::unseparated_literal_suffix,
clippy::used_underscore_binding
)]
// It is often more clear to show that nothing is being moved.
#![allow(clippy::match_ref_pats)]
// Subjective style.
#![allow(
clippy::len_without_is_empty,
clippy::redundant_field_names,
clippy::too_many_arguments
)]
// Default isn't as big a deal as people seem to think it is.
#![allow(clippy::new_without_default, clippy::new_ret_no_self)]
// Arc<Mutex> can be more clear than needing to grok Orderings:
#![allow(clippy::mutex_atomic)]

// TODO: Maybe rename to JVMProcessMap, to decouple from the Nailgun solution?
// Maybe make more general, to manage any long-running process.
use bytes::Bytes;
use std::collections::HashMap;
use sysinfo::{Pid, SystemExt, ProcessExt};

use hashing::Fingerprint;
use process_execution::ExecuteProcessRequest;
use std::path::{PathBuf, Path, Component};
use std::fs::{metadata, File};
use std::{fs, io};
use std::io::Write;

// TODO: This can be just an enum, but using an enum while developing.
type NailgunProcessName = String;
type NailgunProcessFingerprint = Fingerprint;
type Port = usize;

struct NailgunProcessMap {
    // TODO: Possibly wrap in a Mutex
    processes: HashMap<NailgunProcessName, NailgunProcessMetadata>,
    system: sysinfo::System,
}

impl NailgunProcessMap {
    fn new() -> Self {
        NailgunProcessMap {
            processes: HashMap::new(),
            system: sysinfo::System::new(),
        }
    }

    pub fn ensure_nailgun_started(&mut self, name: NailgunProcessName, startup_options: ExecuteProcessRequest) {
        unimplemented!()
    }
}

type NailgunProcessMetadata = (NailgunProcessFingerprint, Pid, Port);
#[derive(Debug)]
pub struct NailgunProcess {
    pid: Pid,
    port: Port,
//    fingerprint: NailgunProcessFingerprint,
    startup_options: ExecuteProcessRequest,
}

const PROCESS_METADATA_DIR: &str = "/tmp/pids/";

#[derive(Copy, Clone)]
enum ProcessMetadataFile {
    Pidfile,
    Portfile
}

impl ProcessMetadataFile {
    fn as_string(self) -> String {
        match self {
            ProcessMetadataFile::Pidfile => String::from("pid"),
            ProcessMetadataFile::Portfile=> String::from("port"),
        }
    }
}

impl NailgunProcess {

    fn get_process_metadata_path(name: &NailgunProcessName, file: &ProcessMetadataFile) -> PathBuf {
        let mut metadata_path = PathBuf::from(PROCESS_METADATA_DIR);
        metadata_path.push(name);
        metadata_path.push(&file.as_string());
        metadata_path
    }

    // TODO: Unify this methid and read_port
    fn read_pid_from_metadata_dir(name: &NailgunProcessName) -> Result<Pid, io::Error> {
        let metadata_path = NailgunProcess::get_process_metadata_path(name, &ProcessMetadataFile::Pidfile);
        println!("Trying to get pid from dir {:?}", metadata_path);
        fs::read_to_string(metadata_path)
          .map(|pid_str| pid_str.parse::<Pid>().unwrap())
    }

    fn read_port_from_metadata_dir(name: &NailgunProcessName) -> Result<Port, io::Error> {
        let metadata_path = NailgunProcess::get_process_metadata_path(name, &ProcessMetadataFile::Portfile);
        println!("Trying to get port from dir {:?}", metadata_path);
        fs::read_to_string(metadata_path)
          .map(|port_str| port_str.parse::<Port>().unwrap())
    }

    fn start_new(name: &NailgunProcessName, startup_options: ExecuteProcessRequest) -> Result<Self, String> {
        println!("I need to start a new process!");
        let cmd = startup_options.argv[0].clone();
        let handle = std::process::Command::new(&cmd)
                                    .args(&startup_options.argv[1..])
                                    .spawn();
        handle
          .map_err(|e| format!("Failed to create child handle {}", e))
          .and_then(|handle| {
              let pid = handle.id() as Pid;
              let pid_path = NailgunProcess::get_process_metadata_path(name, &ProcessMetadataFile::Pidfile);
              let mut pid_file = File::create(pid_path);
              let a = pid_file
                .map_err(|e| format!("Failed to create Pidfile {}", e))
                .and_then(|mut file| {
                    file.write_all(&format!("{}", pid).as_bytes())
                      .map_err(|e| format!("Failed to write pidfile! {}", e))
                      .and_then(|_| Ok(NailgunProcess {
                          pid: pid,
                          port: 1234,
                          startup_options: startup_options
                      }))
                });
              a
          })
    }

    pub fn connect(name: &NailgunProcessName, startup_options: ExecuteProcessRequest, system: &sysinfo::System) -> Result<Self, String> {
        // Read .pids directory with the correct name.
        let maybe_pid = NailgunProcess::read_pid_from_metadata_dir(name);
        if let Ok(pid) = maybe_pid {
            if let Some(process) = system.get_process(pid) {
                println!("I have found process {} for name {}, with cmd {:?}", process.name(), name, process.cmd());
                // Check if the command line has the same shape as the one of the process with the pid.
                if startup_options.argv == process.cmd() {
                    // If it has, fill in the metadata and return the object.
                    println!("The options coincide!");
                    Ok(NailgunProcess {
                        pid: pid,
                        port: NailgunProcess::read_port_from_metadata_dir(name).expect("asdfasdf"),
                        startup_options: startup_options,
                    })
                } else {
                    // The running process doesn't coincide with the options we want.
                    // Restart it.
                    Err(format!("The options for process {} are different to the startup_options! \n Startup Options: {:?}\n Process Cmd: {:?} \n Process Environ {:?}",
                                process.name(), startup_options, process.cmd(), process.environ()
                    ))
                }
            } else {
                panic!("This happens when the process is not running, but there is a metadata file.")
            }
        } else {
            // We couldn't read the Pidfile
            NailgunProcess::start_new(name, startup_options)
        }
    }

}
