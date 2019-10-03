// Copyright 2019 Pants project contributors (see CONTRIBUTORS.md).
// Licensed under the Apache License, Version 2.0 (see LICENSE).

// TODO: Maybe rename to JVMProcessMap, to decouple from the Nailgun solution?
// Maybe make more general, to manage any long-running process.
use bytes::Bytes;
use std::collections::HashMap;

use hashing::Fingerprint;
use crate::ExecuteProcessRequest;
use std::path::{PathBuf, Path, Component};
use std::hash::{Hash, Hasher};
use std::fs::{metadata, File};
use std::{fs, io};
use std::io::{BufRead, BufReader};
use std::io::Write;
use std::collections::hash_map::DefaultHasher;
use std::process::Stdio;
use regex::Regex;
use lazy_static::lazy_static;
use std::sync::Arc;
use parking_lot::Mutex;

lazy_static! {
    static ref NAILGUN_PORT_REGEX: Regex = Regex::new(r".*\s+port\s+(\d+)\.$").unwrap();
}

// TODO: This can be just an enum, but using an enum while developing.
type NailgunProcessName = String;
//type NailgunProcessFingerprint = Fingerprint;
type NailgunProcessFingerprint = u64;
type Pid = usize;
type Port = usize;

pub struct NailgunProcessMap {
    processes: Mutex<HashMap<NailgunProcessName, NailgunProcessMetadata>>,
}

fn hacky_hash(epr: &ExecuteProcessRequest) -> NailgunProcessFingerprint {
    // TODO Use CommandRunner.digest here!
    let mut hasher = DefaultHasher::new();
    epr.hash(&mut hasher);
    hasher.finish()
}

impl NailgunProcessMap {
    pub fn new() -> Self {
        NailgunProcessMap {
            processes: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &NailgunProcessName) -> Option<NailgunProcessMetadata> {
        self.processes.lock().get(name).map(|elem| elem.clone())
    }

    pub fn connect(&self, name: NailgunProcessName, startup_options: ExecuteProcessRequest) -> Result<(), String> {
        // If the process is in the map, check if it's alive using the handle.
        let status = {
            self.processes.lock()
                .get_mut(&name)
                .map(|process| {
                    process.handle.lock().try_wait().map_err(|e| format!("Error getting the process status! {}", e)).clone()
                })
        };
        if let Some(status) = status {
            let (process_name, process_fingerprint, process_pid) = {
                self.processes.lock()
                            .get(&name)
                            .map(|process| { (process.name.clone(), process.fingerprint.clone(), process.pid) })
                            .unwrap()
            };
            println!("Checking if process {} is still alive...", process_pid);
            status
                .map_err(|e| format!("Error reading process status {}", e))
                .and_then(|status| {
                    match status {
                        None => {
                            // Process hasn't exited yet
                            println!("I have found process {}, with fingerprint {:?}",
                                     &name, process_fingerprint);
                            // Check if the command line has the same shape as the one of the process with the pid.
                            let requested_fingerprint = hacky_hash(&startup_options);
                            if requested_fingerprint == process_fingerprint {
                                // If it has, fill in the metadata and return the object.
                                println!("The fingerprints coincide!");
                                Ok(())
                            } else {
                                // The running process doesn't coincide with the options we want.
                                // Restart it.
                                println!("The options for process {} are different to the startup_options! \n Startup Options: {:?}\n Process Cmd: {:?}",
                                         &process_name, startup_options, process_fingerprint
                                );
                                // self.processes.remove(&name);
                                self.start_new_nailgun(name, startup_options)
                            }
                        },
                        _ => {
                            // Process Exited successfully, we need to restart
                            println!("This happens when the process is not running, but there is metadata stored in the map. Restarting process...");
                            // self.processes.remove(&name);
                            self.start_new_nailgun(name, startup_options)
                        }
                    }
                })
        } else {
            // We don't have a running nailgun
            self.start_new_nailgun(name, startup_options)
        }
    }

    fn start_new_nailgun(&self, name: String, startup_options: ExecuteProcessRequest) -> Result<(), String> {
        println!("Starting new Nailgun for {}, with options {:?}", &name, &startup_options);
        NailgunProcessMetadata::start_new(name.clone(), startup_options)
            .and_then(move |process| {
                self.processes.lock().insert(name.clone(), process);
                Ok(())
            })
    }
}

#[derive(Debug, Clone)]
pub struct NailgunProcessMetadata {
    pub name: NailgunProcessName,
    pub fingerprint: NailgunProcessFingerprint, 
    pub pid: Pid, 
    pub port: Port,
    pub handle: Arc<Mutex<std::process::Child>>,
}

fn read_port(child: &mut std::process::Child) -> Result<Port, String> {
    let stdout = child.stdout.as_mut().ok_or(format!("No Stdout found!"));
    stdout.and_then(|stdout| {
        let reader = io::BufReader::new(stdout);
        let line = reader.lines().next().expect("TODO").expect("TODO");
        println!("Read line {}", line);
        let port = &NAILGUN_PORT_REGEX.captures_iter(&line).next().expect("TODO")[1];
        println!("Parsed port is {}", &port);
        port.parse::<Port>()
            .map_err(|e| format!("Error parsing port {}! {}", &port, e))
    })
}

impl NailgunProcessMetadata {
    fn start_new(name: NailgunProcessName, startup_options: ExecuteProcessRequest) -> Result<NailgunProcessMetadata, String> {
       println!("I need to start a new process!");
       let cmd = startup_options.argv[0].clone();
       let stderr_file = File::create(&format!("stderr_{}.txt", name)).unwrap();
       println!("Starting process with cmd: {:?}, args {:?}", cmd, &startup_options.argv[1..]);
        let handle = std::process::Command::new(&cmd)
                                   .current_dir("/Users/bescobar/workspace/otherpants")
                                   .args(&startup_options.argv[1..])
                                   .stdout(Stdio::piped())
                                //    .stderr(Stdio::piped())
                                   .stderr(Stdio::from(stderr_file))
                                   .stdin(Stdio::null())
                                   .spawn();
        handle
          .map_err(|e| format!("Failed to create child handle {}", e))
          .and_then(|mut child| {
              let port = read_port(&mut child);
              port.map(|port| (child, port))
          })
          .and_then(|(child, port)| {
            println!("Created child process: {:?}", child);
            Ok(NailgunProcessMetadata {
                pid: child.id() as Pid,
                port: port,
                fingerprint: hacky_hash(&startup_options),
                name: name,
                handle: Arc::new(Mutex::new(child)),
            })
          })
    }
}

impl Drop for NailgunProcessMetadata {
    fn drop(&mut self) {
        println!("Exiting process {:?}", self);
        self.handle.lock().kill();
    }
}
