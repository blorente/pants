
use super::CommandRunner;
use parking_lot::Mutex;
use crate::{MultiPlatformExecuteProcessRequest, FallibleExecuteProcessResult, ExecuteProcessRequest};
use workunit_store::WorkUnitStore;
use boxfuture::{BoxFuture, Boxable};
use std::path::PathBuf;
use std::sync::Arc;
use futures::future::Future;

pub mod nailgun_process_map;

pub type NailgunProcessMap = nailgun_process_map::NailgunProcessMap;

pub struct NailgunCommandRunner {
    inner: Arc<Box<dyn CommandRunner>>,
    nailguns: NailgunProcessMap,
}

impl NailgunCommandRunner {
    pub fn new(runner: Box<dyn CommandRunner>) -> Self {
        NailgunCommandRunner {
            inner: Arc::new(runner),
            nailguns: NailgunProcessMap::new(),
        }
    }
}

fn is_client_arg(arg: &String) -> bool {
    arg.starts_with("@")
}

impl super::CommandRunner for NailgunCommandRunner {
    fn run(
        &self,
        req: MultiPlatformExecuteProcessRequest,
        workunit_store: WorkUnitStore) -> BoxFuture<FallibleExecuteProcessResult, String> {

        // HACK Transform the nailgun req into nailgun startup args by heuristically transforming the request
        let mut client_req = self.extract_compatible_request(&req).unwrap();
        let mut nailgun_req = self.extract_compatible_request(&req).unwrap();

        nailgun_req.argv.retain(|arg| !is_client_arg(arg));

        let nailgun_name = nailgun_req.argv.last().unwrap().clone(); // We assume the last one is the main class name
        let res = self.nailguns.connect(nailgun_name.clone(), nailgun_req).and_then(|res| {
            let metadata = self.nailguns.get(&nailgun_name).unwrap();
            client_req.argv.retain(is_client_arg);
            client_req.jdk_home = Some(PathBuf::from("~/workspace/bescobar/otherpants/mock_jdk"));
            client_req.env.insert("NAILGUN_PORT".into(), metadata.port.to_string());

            println!("Running Client EPR {:#?} on Nailgun", client_req);
            let res = self.inner.run(MultiPlatformExecuteProcessRequest::from(client_req), workunit_store);
            res.wait()
        });
        futures::done(res).to_boxed()
    }

    fn extract_compatible_request(&self, req: &MultiPlatformExecuteProcessRequest) -> Option<ExecuteProcessRequest> {
        self.inner.extract_compatible_request(req)
    }
}
