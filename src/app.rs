use crate::config::*;
use crate::exec::*;
use crate::prelude::*;

#[async_trait]
trait Component {
    type Output;

    async fn handle(&self) -> Self::Output;
}

struct ConfigExtractor;

#[async_trait]
impl Component for ConfigExtractor {
    type Output = Config;

    async fn handle(&self) -> Self::Output {
        argh::from_env()
    }
}

struct CmdExecutor {
    config: Config,
    executor: Arc<dyn PipedCmdExecutor>,
}

#[async_trait]
impl Component for CmdExecutor {
    type Output = Result<()>;

    async fn handle(&self) -> Self::Output {
        let exit = self.executor.piped_exec(self.config.command.as_str()).await;
        exit.map(|_| ())
    }
}

struct WaitSec {
    config: Config,
}

#[async_trait]
impl Component for WaitSec {
    type Output = ();

    async fn handle(&self) -> Self::Output {
        tokio::time::delay_for(tokio::time::Duration::from_secs_f64(self.config.interval)).await
    }
}

struct StateMachine<C> {
    executor: Arc<dyn PipedCmdExecutor>,
    execution_count: usize,
    component: C,
}

impl<C> StateMachine<C> {
    fn count_up(self) -> Self {
        StateMachine {
            execution_count: self.execution_count + 1,
            ..self
        }
    }
}

#[async_trait]
impl<T: 'static, C: Component<Output = T> + Send + Sync> Component for StateMachine<C> {
    type Output = T;

    async fn handle(&self) -> Self::Output {
        self.component.handle().await
    }
}

pub enum Transition<T> {
    Next(T),
    Done,
}

#[async_trait]
pub trait Transform: Sized {
    async fn handle(self) -> Transition<Self>;
}

pub enum App {
    ParseOption(StateMachine<ConfigExtractor>),
    ExecuteCommand(StateMachine<CmdExecutor>),
    Sleep(StateMachine<WaitSec>),
}

impl App {
    pub fn new(executor: impl PipedCmdExecutor + 'static) -> Self {
        App::ParseOption(StateMachine {
            executor: Arc::new(executor),
            component: ConfigExtractor,
            execution_count: 0,
        })
    }
}

impl From<(StateMachine<ConfigExtractor>, Config)> for StateMachine<CmdExecutor> {
    fn from((machine, config): (StateMachine<ConfigExtractor>, Config)) -> Self {
        Self {
            component: CmdExecutor {
                config,
                executor: machine.executor.clone(),
            },
            executor: machine.executor,
            execution_count: machine.execution_count,
        }
    }
}

impl From<StateMachine<CmdExecutor>> for StateMachine<WaitSec> {
    fn from(machine: StateMachine<CmdExecutor>) -> Self {
        Self {
            component: WaitSec {
                config: machine.component.config,
            },
            executor: machine.executor,
            execution_count: machine.execution_count,
        }
    }
}

impl From<StateMachine<WaitSec>> for StateMachine<CmdExecutor> {
    fn from(machine: StateMachine<WaitSec>) -> Self {
        Self {
            component: CmdExecutor {
                config: machine.component.config,
                executor: machine.executor.clone(),
            },
            executor: machine.executor,
            execution_count: machine.execution_count,
        }
    }
}

#[async_trait]
impl Transform for App {
    async fn handle(self) -> Transition<Self> {
        match self {
            App::ParseOption(machine) => machine
                .handle()
                .await
                .apply(|config| App::ExecuteCommand((machine, config).into()))
                .apply(Transition::Next),
            App::ExecuteCommand(machine) => {
                let _ = machine.handle().await;
                let machine = machine.count_up();

                match machine.component.config.count {
                    Some(max) if machine.execution_count >= max => Transition::Done,
                    _ => Transition::Next(App::Sleep(machine.into())),
                }
            }
            App::Sleep(machine) => machine
                .handle()
                .await
                .apply(|_| App::ExecuteCommand(machine.into()))
                .apply(Transition::Next),
        }
    }
}

pub async fn run<T: Transform>(mut app: T) {
    loop {
        match app.handle().await {
            Transition::Next(next) => app = next,
            Transition::Done => break,
        }
    }
}
