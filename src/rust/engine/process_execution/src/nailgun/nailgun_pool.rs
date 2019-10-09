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
use log::{debug, info, trace};
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
        trace!("Locking nailgun process pool so that only one can be connecting at a time.");
        let mut processes = self.processes.lock();
        // If the process is in the map, check if it's alive using the handle.
        let status = {
            processes
                .get_mut(&name)
                .map(|process| {
                    process.handle
                        .lock()
                        .try_wait()
                        .map_err(|e| format!("Error getting the process status! {}", e))
                        .clone()
                })
        };
        let connection_result = if let Some(status) = status {
            let (process_name, process_fingerprint, process_port) = {
                processes
                    .get(&name)
                    .map(|process| { (process.name.clone(), process.fingerprint.clone(), process.port) })
                    .unwrap()
            };
            debug!("Checking if nailgun server {} is still alive at port {}...", &process_name, process_port);
            status
                .map_err(|e| format!("Error reading nailgun server status {}", e))
                .and_then(|status| {
                    match status {
                        None => {
                            // Process hasn't exited yet
                            debug!("Found nailgun process {}, with fingerprint {:?}",
                                     &name, process_fingerprint);
                            if nailgun_req_digest == process_fingerprint {
                                debug!("The fingerprint of the running nailgun {:?} coincides with the requested fingerprint {:?}. Connecting to existing server.",
                                        nailgun_req_digest, process_fingerprint);
                                Ok(process_port)
                            } else {
                                // The running process doesn't coincide with the options we want.
                                // Restart it.
                                debug!("The options for process {} are different to the startup_options! \n Startup Options: {:?}\n Process Cmd: {:?}",
                                         &process_name, startup_options, process_fingerprint
                                );
                                // self.processes.remove(&name);
                                self.start_new_nailgun(&mut *processes, name, startup_options, workdir_path, nailgun_req_digest)
                            }
                        },
                        _ => {
                            debug!("The requested nailgun server was not running anymore. Restarting process...");
                            self.start_new_nailgun(&mut *processes, name, startup_options, workdir_path, nailgun_req_digest)
                        }
                    }
                })
        } else {
            // We don't have a running nailgun
            self.start_new_nailgun(&mut *processes, name, startup_options, workdir_path, nailgun_req_digest)
        };
        trace!("Unlocking nailgun process pool.");
        connection_result
    }

    fn start_new_nailgun(&self,
                         processes: &mut NailgunProcessMap,
                         name: String,
                         startup_options: ExecuteProcessRequest,
                         workdir_path: &PathBuf,
                         nailgun_req_digest: Digest) -> Result<Port, String> {
        debug!("Starting new nailgun server for {}, with options {:?}", &name, &startup_options);
        NailgunProcessMetadata::start_new(name.clone(), startup_options, workdir_path, nailgun_req_digest)
            .and_then(move |process| {
                let port = process.port;
                processes.insert(name.clone(), process);
                Ok(port)
            })
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
        let line = reader.lines().next()
            .ok_or("There is no line ready in the child's output")?
            .map_err(|err| format!("{}", err))?;
        let port = &NAILGUN_PORT_REGEX.captures_iter(&line).next()
            .ok_or("Output for nailgun server didn't match the regex!")?[1];
        port.parse::<Port>()
            .map_err(|e| format!("Error parsing port {}! {}", &port, e))
    })
}

impl NailgunProcessMetadata {
    fn start_new(name: NailgunProcessName, startup_options: ExecuteProcessRequest, workdir_path: &PathBuf, nailgun_req_digest: Digest) -> Result<NailgunProcessMetadata, String> {
        let cmd = startup_options.argv[0].clone();
        debug!("Starting new nailgun server with cmd: {:?}, args {:?}, in cwd {:?}", cmd, &startup_options.argv[1..], &workdir_path);
        let handle = std::process::Command::new(&cmd)
                .args(&startup_options.argv[1..])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(&workdir_path)
                .spawn();
        handle
          .map_err(|e| format!("Failed to create child handle with cmd: {} options {:#?}: {}", &cmd, &startup_options, e))
          .and_then(|mut child| {
              let port = read_port(&mut child);
              port.map(|port| (child, port))
          })
          .and_then(|(child, port)| {
            debug!("Created nailgun server process with pid {} and port {}", child.id(), port);
            Ok(NailgunProcessMetadata {
                pid: child.id() as Pid,
                port: port,
                fingerprint: nailgun_req_digest,
                name: name,
                handle: Arc::new(Mutex::new(child)),
            })
          })
    }
}

impl Drop for NailgunProcessMetadata {
    fn drop(&mut self) {
        debug!("Exiting nailgun server process {:?}", self);
        self.handle.lock().kill();
    }
}
