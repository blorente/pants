
use super::CommandRunner;
use parking_lot::Mutex;
use crate::{MultiPlatformExecuteProcessRequest, FallibleExecuteProcessResult, ExecuteProcessRequest, Platform};
use workunit_store::WorkUnitStore;
use boxfuture::{try_future, BoxFuture, Boxable};
use std::path::PathBuf;
use std::sync::Arc;
use futures::future::Future;
use std::os::unix::fs::symlink;
use std::collections::btree_map::BTreeMap;
use std::collections::btree_set::BTreeSet;
use std::time::Duration;

pub mod nailgun_process_map;

pub type NailgunProcessMap = nailgun_process_map::NailgunProcessMap;

pub struct NailgunCommandRunner {
    inner: Arc<super::local::CommandRunner>,
    nailguns: NailgunProcessMap,
}

impl NailgunCommandRunner {
    pub fn new(runner: super::local::CommandRunner) -> Self {
        NailgunCommandRunner {
            inner: Arc::new(runner),
            nailguns: NailgunProcessMap::new(),
        }
    }
}

fn is_client_arg(arg: &String) -> bool {
    arg.starts_with("@")
}

fn get_nailgun_request(classpath: String) -> ExecuteProcessRequest {
    ExecuteProcessRequest {
        argv: vec![
            String::from("/Library/Java/JavaVirtualMachines/TwitterJDK/Contents/Home/bin/java"),
            String::from("-Xmx1g"),
            String::from("-Dpants.buildroot=/Users/bescobar/workspace/otherpants"),
            String::from("-Dpants.nailgun.owner=/Users/bescobar/workspace/otherpants/.pants.d/ng/FindBugs_compile_findbugs"),
            String::from("-Dpants.nailgun.fingerprint=a89c6538bef5aabf182b06a81f910a66c87d28eb"),
            String::from("-cp"),
            format!("/Users/bescobar/workspace/otherpants/.pants.d/bootstrap/bootstrap-jvm-tools/a0ebe8e0b001/ivy/jars/com.martiansoftware/nailgun-server/jars/nailgun-server-0.9.1.jar:{}", classpath),
            String::from("com.martiansoftware.nailgun.NGServer"),
            String::from(":0")
        ],
        env: BTreeMap::new(),
        input_files: Default::default(),
        output_files: BTreeSet::new(),
        output_directories: BTreeSet::new(),
        timeout: Duration::new(1000, 0),
        description: String::from("EPR to start a nailgun"),
        jdk_home: None,
        target_platform: Platform::Darwin,
    }
}

fn extract_main_class(nailgun_args: &Vec<String>) -> String {
    nailgun_args.last().unwrap().clone()
}

fn extract_classpath(nailgun_args: &Vec<String>) -> String {
    let mut it = nailgun_args.into_iter();
    while it.next().unwrap() != "-cp" { };
    it.next().unwrap().clone()
}


impl super::CommandRunner for NailgunCommandRunner {
    fn run(
        &self,
        req: MultiPlatformExecuteProcessRequest,
        workunit_store: WorkUnitStore) -> BoxFuture<FallibleExecuteProcessResult, String> {

        let workdir_path = PathBuf::from("/tmp/a");
        let workdir_path2 = workdir_path.clone();

        // HACK Transform the nailgun req into nailgun startup args by heuristically transforming the request
        let mut client_req = self.extract_compatible_request(&req).unwrap();
        let nailgun_args: Vec<String> = client_req.argv.clone().into_iter().filter(|elem| !is_client_arg(elem)).collect();
        let nailgun_name = nailgun_args.last().unwrap().clone(); // We assume the last one is the main class name
        let custom_classpath = extract_classpath(&nailgun_args);
        let nailgun_req = get_nailgun_request(custom_classpath);

        let custom_main_class = extract_main_class(&nailgun_args);
        let maybe_jdk_home = nailgun_req.jdk_home.clone();

        let nailgun_name = nailgun_req.argv.last().unwrap().clone(); // We assume the last one is the main class name
        let materialize = self.inner
            .store
            .materialize_directory(workdir_path.clone(), nailgun_req.input_files, workunit_store.clone())
            .and_then(move |_metadata| {
                maybe_jdk_home.map_or(Ok(()), |jdk_home| {
                    symlink(jdk_home, workdir_path2.clone().join(".jdk"))
                        .map_err(|err| format!("Error making symlink for local execution: {:?}", err))
                })?;
                Ok(())
            })
            .wait();

        println!("Materialized directory! {:?}", &workdir_path);

        let nailgun = materialize
            .and_then(|_metadata| {
                self.nailguns
                    .connect(nailgun_name.clone(), nailgun_req, &workdir_path)
            });

        println!("Connected to nailgun!");

        pause();

        let res = nailgun
            .and_then(|res| {
                let metadata = self.nailguns.get(&nailgun_name).unwrap();
                let client_args: Vec<String> = client_req.argv.clone().into_iter().filter(is_client_arg).collect();

                client_req.argv = vec![
                    ".jdk/bin/java".to_string(),
                    custom_main_class,
                ];
                client_req.argv.extend(client_args);
                client_req.jdk_home = Some(PathBuf::from("/Users/bescobar/workspace/otherpants/mock_jdk"));
                client_req.env.insert("NAILGUN_PORT".into(), metadata.port.to_string());

                println!("Running Client EPR {:#?} on Nailgun", client_req);
                let res = self.inner.run(MultiPlatformExecuteProcessRequest::from(client_req), workunit_store);
                res.wait()
            });

        println!("Done executing hte child process! {:#?}", res);

        futures::future::done(res).to_boxed()
    }

    fn extract_compatible_request(&self, req: &MultiPlatformExecuteProcessRequest) -> Option<ExecuteProcessRequest> {
        self.inner.extract_compatible_request(req)
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

