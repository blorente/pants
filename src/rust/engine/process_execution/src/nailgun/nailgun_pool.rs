// Copyright 2019 Pants project contributors (see CONTRIBUTORS.md).
// Licensed under the Apache License, Version 2.0 (see LICENSE).

// TODO: Maybe rename to JVMProcessMap, to decouple from the Nailgun solution?
// Maybe make more general, to manage any long-running process.
use bytes::Bytes;
use std::collections::HashMap;

use hashing::{Fingerprint, Digest};
use crate::{ExecuteProcessRequest, ExecuteProcessRequestMetadata, MultiPlatformExecuteProcessRequest};
use std::path::{PathBuf, Path, Component};
use std::hash::{Hash, Hasher};
use std::fs::{metadata, File, read};
use std::{fs, io};
use std::io::{BufRead, BufReader, Stdout, Read};
use std::io::Write;
use std::collections::hash_map::DefaultHasher;
use std::process::Stdio;
use regex::Regex;
use lazy_static::lazy_static;
use std::sync::Arc;
use parking_lot::Mutex;
use crate::local::StreamedHermeticCommand;
use log::info;
use core::borrow::BorrowMut;

lazy_static! {
    static ref NAILGUN_PORT_REGEX: Regex = Regex::new(r".*\s+port\s+(\d+)\.$").unwrap();
}

pub type NailgunProcessName = String;
type NailgunProcessFingerprint = Digest;
type NailgunProcessMap = HashMap<NailgunProcessName, NailgunProcessMetadata>;
type Pid = usize;
type Port = usize;

#[derive(Clone)]
pub struct NailgunPool {
    processes: Arc<Mutex<NailgunProcessMap>>,
}

impl NailgunPool {
    pub fn new() -> Self {
        NailgunPool {
            processes: Arc::new(Mutex::new(NailgunProcessMap::new())),
        }
    }

    pub fn connect(&self,
                   name: NailgunProcessName,
                   startup_options: ExecuteProcessRequest,
                   workdir_path: &PathBuf,
                   nailgun_req_digest: Digest) -> Result<Port, String> {
        info!("BL: Locking processes so that only one can be connecting at a time");
        let mut processes = self.processes.lock();
        // If the process is in the map, check if it's alive using the handle.
        let status = {
            processes
                .get_mut(&name)
                .map(|process| {
                    process.handle.lock().try_wait().map_err(|e| format!("Error getting the process status! {}", e)).clone()
                })
        };
        let a = if let Some(status) = status {
            let (process_name, process_fingerprint, process_port) = {
                processes
                    .get(&name)
                    .map(|process| { (process.name.clone(), process.fingerprint.clone(), process.port) })
                    .unwrap()
            };
            info!("Checking if process {} is still alive at port {}...", &process_name, process_port);
            status
                .map_err(|e| format!("Error reading process status {}", e))
                .and_then(|status| {
                    match status {
                        None => {
                            // Process hasn't exited yet
                            info!("I have found process {}, with fingerprint {:?}",
                                     &name, process_fingerprint);
                            // Check if the command line has the same shape as the one of the process with the pid.
                            if nailgun_req_digest == process_fingerprint {
                                // If it has, fill in the metadata and return the object.
                                info!("The fingerprints coincide!");
                                Ok(process_port)
                            } else {
                                // The running process doesn't coincide with the options we want.
                                // Restart it.
                                info!("The options for process {} are different to the startup_options! \n Startup Options: {:?}\n Process Cmd: {:?}",
                                         &process_name, startup_options, process_fingerprint
                                );
                                // self.processes.remove(&name);
                                self.start_new_nailgun(&mut *processes, name, startup_options, workdir_path, nailgun_req_digest)
                            }
                        },
                        _ => {
                            // Process Exited successfully, we need to restart
                            info!("This happens when the process is not running, but there is metadata stored in the map. Restarting process...");
                            // self.processes.remove(&name);
                            self.start_new_nailgun(&mut *processes, name, startup_options, workdir_path, nailgun_req_digest)
                        }
                    }
                })
        } else {
            // We don't have a running nailgun
            self.start_new_nailgun(&mut *processes, name, startup_options, workdir_path, nailgun_req_digest)
        };
        info!("BL: About to unlock!");
        a
    }

    fn start_new_nailgun(&self,
                         processes: &mut NailgunProcessMap,
                         name: String,
                         startup_options: ExecuteProcessRequest,
                         workdir_path: &PathBuf,
                         nailgun_req_digest: Digest) -> Result<Port, String> {
        info!("Starting new Nailgun for {}, with options {:?}", &name, &startup_options);
        NailgunProcessMetadata::start_new(name.clone(), startup_options, workdir_path, nailgun_req_digest)
            .and_then(move |process| {
                let port = process.port;
                processes.insert(name.clone(), process);
                Ok(port)
            })
    }

    pub fn print_stdout(&self, name: &NailgunProcessName) -> String {
        self.processes.lock()
            .get_mut(name)
            .map(|process| process.print_stdout()).unwrap()
    }
}

#[derive(Debug)]
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
        let line = reader.lines().next().expect("There is no line ready in the child's output").expect("Failed to read element");
        info!("Read line {}", line);
        let port = &NAILGUN_PORT_REGEX.captures_iter(&line).next().expect("I didn't match the output!")[1];
        info!("Parsed port is {}", &port);
        port.parse::<Port>()
            .map_err(|e| format!("Error parsing port {}! {}", &port, e))
    })
}

impl NailgunProcessMetadata {
    fn start_new(name: NailgunProcessName, startup_options: ExecuteProcessRequest, workdir_path: &PathBuf, nailgun_req_digest: Digest) -> Result<NailgunProcessMetadata, String> {
        info!("I need to start a new process!");
        info!("I need to start a new process!");
        info!("I need to start a new process!");
        info!("I need to start a new process!");
        info!("I need to start a new process!");
        info!("I need to start a new process!");
        info!("I need to start a new process!");
        info!("I need to start a new process!");
        let cmd = startup_options.argv[0].clone();
        let stderr_file = File::create(&format!("stderr_{}.txt", name)).unwrap();
        info!("Starting process with cmd: {:?}, args {:?}, in cwd {:?}", cmd, &startup_options.argv[1..], &workdir_path);
        let handle = std::process::Command::new(&cmd)
                .args(&startup_options.argv[1..])
                .stdout(Stdio::piped())
//                .stderr(Stdio::piped())
                .stderr(stderr_file)
                .current_dir(&workdir_path)
                .spawn();
        handle
          .map_err(|e| format!("Failed to create child handle with cmd: {} options {:#?}: {}", &cmd, &startup_options, e))
          .and_then(|mut child| {
              let port = read_port(&mut child);
              port.map(|port| (child, port))
          })
          .and_then(|(child, port)| {
            info!("Created nailgun server process with pid {}: {:?}", child.id(), child);
            Ok(NailgunProcessMetadata {
                pid: child.id() as Pid,
                port: port,
                fingerprint: nailgun_req_digest,
                name: name,
                handle: Arc::new(Mutex::new(child)),
            })
          })
    }

    fn print_stdout(&mut self) -> String {
        let mut handle =  self.handle.lock();
        let mut stdout = handle.stdout.as_mut().unwrap();
        let mut s = String::new();

        stdout.read_to_string(&mut s);
        info!("BL: STDOUT OF THE SERVER: {}", s);
        s
    }
}

impl Drop for NailgunProcessMetadata {
    fn drop(&mut self) {
        info!("Exiting process {:?}", self);
        info!("Exiting process {:?}", self);
        info!("Exiting process {:?}", self);
        info!("Exiting process {:?}", self);
        info!("Exiting process {:?}", self);
        info!("Exiting process {:?}", self);
        info!("Exiting process {:?}", self);
        info!("Exiting process {:?}", self);
        info!("Exiting process {:?}", self);

        self.handle.lock().kill();
    }
}
