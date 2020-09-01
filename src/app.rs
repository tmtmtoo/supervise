mod components;

use crate::exec::*;
use crate::prelude::*;
use components::*;

#[async_trait]
pub trait Component {
    type Output;

    async fn handle(&self) -> Self::Output;
}

pub enum Transition<T> {
    Next(T),
    Done,
}

#[async_trait]
pub trait StateMachine: Sized {
    async fn handle(self) -> Transition<Self>;
}

enum AppState<E, S> {
    ExecuteCommand(E),
    Sleep(S),
}

pub struct App<E, S> {
    state: AppState<E, S>,
    count: usize,
    limit: Option<usize>,
}

impl App<SharedState<PrintableCmdNotFound<CmdExecutor>>, SharedState<WaitSec>> {
    pub fn new(command: String, limit: Option<usize>, interval: f64) -> Self {
        let executor = Arc::new(TokioPipedCmdExecutor::new());

        Self {
            state: AppState::ExecuteCommand(SharedState::new(
                command.to_owned(),
                interval,
                executor.clone(),
                PrintableCmdNotFound::new(command.to_owned(), CmdExecutor::new(command, executor)),
            )),
            count: 0,
            limit,
        }
    }
}

#[async_trait]
impl<E, S> StateMachine for App<E, S>
where
    E: Component<Output = Result<()>> + Into<S> + Send + Sync,
    S: Component<Output = ()> + Into<E> + Send + Sync,
{
    async fn handle(self) -> Transition<Self> {
        match self.state {
            AppState::ExecuteCommand(component) => {
                let _ = component.handle().await;
                let next_count = self.count + 1;

                match self.limit {
                    Some(limit) if next_count >= limit => Transition::Done,
                    _ => Transition::Next(App {
                        state: AppState::Sleep(component.into()),
                        count: next_count,
                        limit: self.limit,
                    }),
                }
            }
            AppState::Sleep(component) => {
                component.handle().await;

                Transition::Next(App {
                    state: AppState::ExecuteCommand(component.into()),
                    ..self
                })
            }
        }
    }
}

pub async fn run<S: StateMachine>(mut app: S) {
    loop {
        match app.handle().await {
            Transition::Next(next) => app = next,
            Transition::Done => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestE;

    #[async_trait]
    impl Component for TestE {
        type Output = Result<()>;
        async fn handle(&self) -> Self::Output {
            Ok(())
        }
    }

    struct TestS;

    #[async_trait]
    impl Component for TestS {
        type Output = ();
        async fn handle(&self) -> Self::Output {
            ()
        }
    }

    impl From<TestE> for TestS {
        fn from(_: TestE) -> Self {
            TestS
        }
    }

    impl From<TestS> for TestE {
        fn from(_: TestS) -> Self {
            TestE
        }
    }

    #[tokio::test]
    async fn exec_cmd_to_sleep_without_limit() {
        let app = App::<TestE, TestS> {
            state: AppState::ExecuteCommand(TestE),
            count: 0,
            limit: None,
        };

        let next = app.handle().await;

        assert!(match &next {
            Transition::Next(a) => match a.state {
                AppState::Sleep(_) => true,
                _ => false,
            },
            _ => false,
        });

        assert_eq!(
            match next {
                Transition::Next(a) => Some(a.count),
                _ => None,
            },
            Some(1)
        );
    }

    #[tokio::test]
    async fn exec_cmd_to_sleep_with_limit() {
        let app = App::<TestE, TestS> {
            state: AppState::ExecuteCommand(TestE),
            count: 0,
            limit: Some(2),
        };

        let next = app.handle().await;

        assert!(match &next {
            Transition::Next(a) => match a.state {
                AppState::Sleep(_) => true,
                _ => false,
            },
            _ => false,
        });

        assert_eq!(
            match next {
                Transition::Next(a) => Some(a.count),
                _ => None,
            },
            Some(1)
        );
    }

    #[tokio::test]
    async fn exec_cmd_to_done() {
        let app = App::<TestE, TestS> {
            state: AppState::ExecuteCommand(TestE),
            count: 0,
            limit: Some(1),
        };

        assert!(match app.handle().await {
            Transition::Done => true,
            _ => false,
        });
    }

    #[tokio::test]
    async fn sleep_to_exec() {
        let app = App::<TestE, TestS> {
            state: AppState::Sleep(TestS),
            count: 0,
            limit: Some(1),
        };

        assert!(match app.handle().await {
            Transition::Next(a) => match a.state {
                AppState::ExecuteCommand(_) => true,
                _ => false,
            },
            _ => false,
        });
    }
}
