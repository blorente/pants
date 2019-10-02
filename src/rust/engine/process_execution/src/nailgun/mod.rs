
use super::CommandRunner;
use std::sync::Arc;
use crate::{MultiPlatformExecuteProcessRequest, FallibleExecuteProcessResult, ExecuteProcessRequest};
use workunit_store::WorkUnitStore;
use futures::Future;
use boxfuture::BoxFuture;

pub mod nailgun_process_map;

pub type NailgunProcessMap = nailgun_process_map::NailgunProcessMap;

struct NailgunCommandRunner {
    inner: Arc<dyn CommandRunner>,
    nailguns: NailgunProcessMap
}

impl NailgunCommandRunner {
    pub fn new(runner: Box<dyn CommandRunner>) -> Self {
        NailgunCommandRunner {
            inner: runner.into(),
            nailguns: NailgunProcessMap::new(),
        }
    }
}

impl super::CommandRunner for NailgunCommandRunner {
    fn run(&self, req: MultiPlatformExecuteProcessRequest, workunit_store: WorkUnitStore) -> BoxFuture<FallibleExecuteProcessResult, String> {
        unimplemented!()
    }

    fn extract_compatible_request(&self, req: &MultiPlatformExecuteProcessRequest) -> Option<ExecuteProcessRequest> {
        unimplemented!()
    }
}
