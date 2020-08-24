use crate::config::*;
use crate::exec::*;
use crate::prelude::*;

#[derive(new)]
pub struct CmdExecutor {
    command: String,
    executor: Arc<dyn PipedCmdExecutor>,
}

#[async_trait]
impl super::Component for CmdExecutor {
    type Output = Result<()>;

    async fn handle(&self) -> Self::Output {
        let exit = self.executor.piped_exec(self.command.as_str()).await;
        exit.map(|_| ())
    }
}

pub struct WaitSec {
    sec: f64,
}

#[async_trait]
impl super::Component for WaitSec {
    type Output = ();

    async fn handle(&self) -> Self::Output {
        tokio::time::delay_for(tokio::time::Duration::from_secs_f64(self.sec)).await
    }
}

#[derive(new)]
pub struct SharedState<C> {
    config: Config,
    executor: Arc<dyn PipedCmdExecutor>,
    inner: C,
}

#[async_trait]
impl<T: 'static, C: super::Component<Output = T> + Send + Sync> super::Component
    for SharedState<C>
{
    type Output = T;

    async fn handle(&self) -> Self::Output {
        self.inner.handle().await
    }
}

impl From<SharedState<CmdExecutor>> for SharedState<WaitSec> {
    fn from(state: SharedState<CmdExecutor>) -> Self {
        Self {
            inner: WaitSec {
                sec: state.config.interval,
            },
            config: state.config,
            executor: state.executor,
        }
    }
}

impl From<SharedState<WaitSec>> for SharedState<CmdExecutor> {
    fn from(state: SharedState<WaitSec>) -> Self {
        Self {
            inner: CmdExecutor {
                command: state.config.command.to_owned(),
                executor: state.executor.clone(),
            },
            config: state.config,
            executor: state.executor,
        }
    }
}
