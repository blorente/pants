
use super::CommandRunner;
use parking_lot::Mutex;
use crate::{MultiPlatformExecuteProcessRequest, FallibleExecuteProcessResult, ExecuteProcessRequest, Platform, ExecuteProcessRequestMetadata};
use workunit_store::WorkUnitStore;
use boxfuture::{try_future, BoxFuture, Boxable};
use std::path::PathBuf;
use std::sync::Arc;
use futures::future::Future;
use std::os::unix::fs::symlink;
use std::collections::btree_map::BTreeMap;
use std::collections::btree_set::BTreeSet;
use std::time::Duration;
use log::{info, debug};
use hashing::Digest;
use crate::nailgun::nailgun_pool::NailgunProcessName;

pub mod nailgun_pool;

pub type NailgunPool = nailgun_pool::NailgunPool;

pub struct NailgunCommandRunner {
    inner: Arc<super::local::CommandRunner>,
    nailguns: NailgunPool,
    metadata: ExecuteProcessRequestMetadata,
    workdir_base: PathBuf,
}


fn is_client_arg(arg: &String) -> bool {
    arg.starts_with("@")
}

/// Represents the result of parsing the args of a nailgunnable ExecuteProcessRequest
/// TODO We may want to split the classpath by the ":", and store it as a Vec<String>
///         to allow for deep fingerprinting.
struct ParsedArgLists {
    nailgun_args: Vec<String>,
    client_args: Vec<String>,
}

static NAILGUN_JAR: &str = "/Users/bescobar/workspace/otherpants/.pants.d/bootstrap/bootstrap-jvm-tools/a0ebe8e0b001/ivy/jars/com.martiansoftware/nailgun-server/jars/nailgun-server-0.9.1.jar";
static NAILGUN_MAIN_CLASS: &str = "com.martiansoftware.nailgun.NGServer";
static ARGS_TO_START_NAILGUN: [&str; 1] = [":0"];

fn split_args(args: &Vec<String>) -> ParsedArgLists {
    let mut iterator = args.iter();
    let mut nailgun_args = vec![];
    let mut client_args = vec![];
    let mut nailgun_classpath = None;
    let mut have_seen_classpath = false;
    let mut have_processed_classpath = false;
    let mut have_seen_main_class = false;
    for arg in args {
        if have_seen_main_class {
            client_args.push(arg.clone());
        } else if (arg == "-cp" || arg =="-classpath") && !have_seen_classpath {
            have_seen_classpath = true;
            nailgun_args.push(arg.clone());
        } else if have_seen_classpath && !have_processed_classpath {
            let formatted_classpath = format!("{}:{}", NAILGUN_JAR, arg);
            nailgun_args.push(formatted_classpath.clone());
            nailgun_classpath = Some(formatted_classpath);
            have_processed_classpath = true;
        } else if have_processed_classpath && !arg.starts_with("-") {
            client_args.push(arg.clone());
            have_seen_main_class = true;
        } else {
            nailgun_args.push(arg.clone());
        }
    }
    ParsedArgLists {
        nailgun_args: nailgun_args,
        client_args: client_args
    }
}

fn get_nailgun_request(args: Vec<String>, input_files: Digest, jdk: Option<PathBuf>) -> ExecuteProcessRequest {
    let mut full_args = args;
    full_args.push(NAILGUN_MAIN_CLASS.to_string());
    full_args.extend(ARGS_TO_START_NAILGUN.iter().map(|a| a.to_string()));

    ExecuteProcessRequest {
        argv: full_args,
        env: BTreeMap::new(),
        input_files: input_files,
        output_files: BTreeSet::new(),
        output_directories: BTreeSet::new(),
        timeout: Duration::new(1000, 0),
        description: String::from("ExecuteProcessRequest to start a nailgun"),
        jdk_home: jdk,
        target_platform: Platform::Darwin,
        is_nailgunnable: true,
    }
}

impl NailgunCommandRunner {
    pub fn new(runner: super::local::CommandRunner, metadata: ExecuteProcessRequestMetadata) -> Self {
        let mut workdir_base = std::env::temp_dir();

        NailgunCommandRunner {
            inner: Arc::new(runner),
            nailguns: NailgunPool::new(),
            metadata: metadata,
            workdir_base: workdir_base,
        }
    }

    fn get_nailguns_workdir(&self, nailgun_name: &NailgunProcessName) -> Result<PathBuf, String> {
        let workdir = self.workdir_base.clone().join(nailgun_name);
        if self.workdir_base.exists() {
            std::fs::create_dir_all(workdir.clone())
                .map_err(|err| format!("Error creating the nailgun workdir! {}", err))
                .map(|_| workdir)
        } else {
            info!("BL: Nailgun workdir {:?} exits! Using that...", self.workdir_base);
            Ok(workdir)
        }
    }
}

impl super::CommandRunner for NailgunCommandRunner {
    fn run(
        &self,
        req: MultiPlatformExecuteProcessRequest,
        workunit_store: WorkUnitStore) -> BoxFuture<FallibleExecuteProcessResult, String> {

        let mut client_req = self.extract_compatible_request(&req).unwrap();
        info!("BL: Full EPR:\n {:#?}", &client_req);
        if !client_req.is_nailgunnable {
            info!("BL: The request is not nailgunnable! Short-circuiting to regular process execution");
            return self.inner.run(req, workunit_store)
        }
        let ParsedArgLists {nailgun_args, client_args } = split_args(&client_req.argv);
        let nailgun_req = get_nailgun_request(nailgun_args, client_req.input_files, client_req.jdk_home.clone());
        info!("BL: NAILGUN EPR:\n {:#?}", &nailgun_req);

        let maybe_jdk_home = nailgun_req.jdk_home.clone();

        let main_class = client_args.iter().next().unwrap().clone(); // We assume the last one is the main class name
        let nailgun_req_digest = crate::digest(MultiPlatformExecuteProcessRequest::from(nailgun_req.clone()), &self.metadata);
        let nailgun_name = format!("{}_{}", main_class, nailgun_req_digest.0);
        let nailgun_name2 = nailgun_name.clone();

        let nailguns_workdir = try_future!(self.get_nailguns_workdir(&nailgun_name));

        let workdir_path2 = nailguns_workdir.clone();
        let workdir_path3= nailguns_workdir.clone();
        let workdir_path4 = nailguns_workdir.clone();
        let materialize = self.inner
            .store
            .materialize_directory(workdir_path.clone(), nailgun_req.input_files, workunit_store.clone())
            .and_then(move |_metadata| {
                maybe_jdk_home.map_or(Ok(()), |jdk_home_relpath| {
                    let mut jdk_home_in_workdir = workdir_path2.clone().join(".jdk");
                    if !jdk_home_in_workdir.exists() {
                        symlink(jdk_home_relpath, jdk_home_in_workdir)
                            .map_err(|err| format!("Error making symlink for local execution in workdir {:?}: {:?}", &workdir_path2, err))
                    } else {
                        debug!("JDK home for Nailgun already exists in {:?}. Using that one.", &workdir_path4);
                        Ok(())
                    }
                })?;
                Ok(())
            })
            .inspect(move |_| info!("Materialized directory! {:?}", &workdir_path3));

        let nailguns = self.nailguns.clone();
        let metadata = self.metadata.clone();
        let nailgun = materialize
            .map(move |_metadata| {
                nailguns.connect(nailgun_name.clone(), nailgun_req, &nailguns_workdir, nailgun_req_digest)
            })
            .inspect(|_| info!("Connected to nailgun!"));

        let inner = self.inner.clone();
        let nailguns = self.nailguns.clone();
        let res = nailgun
            .and_then(move |res| {
                match res {
                    Ok(port) => {
                        info!("Got nailgun at port {:#?}", port);

                        client_req.argv = vec![
                            ".jdk/bin/java".to_string(),
                        ];
                        client_req.argv.extend(client_args);
                        client_req.jdk_home = Some(PathBuf::from("/Users/bescobar/workspace/otherpants/mock_jdk"));
                        client_req.env.insert("NAILGUN_PORT".into(), port.to_string());

                        info!("Running Client EPR {:#?} on Nailgun", client_req);
                        inner.run(MultiPlatformExecuteProcessRequest::from(client_req), workunit_store)
                    }
                    Err(e) => {
                        futures::future::err(e).to_boxed()
                    }
                }
            });

        res.to_boxed()
    }

    fn extract_compatible_request(&self, req: &MultiPlatformExecuteProcessRequest) -> Option<ExecuteProcessRequest> {
        self.inner.extract_compatible_request(req)
    }
}
