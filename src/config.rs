use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt, PartialEq)]
#[structopt(author = "")]
#[structopt(raw(setting = "AppSettings::AllowLeadingHyphen"))]
/// Supervise command execution.
pub struct Config {
    /// maximum number of executions
    #[structopt(short, long)]
    pub count: Option<usize>,

    /// execution interval (sec)
    #[structopt(short, long, default_value = "0.1")]
    pub interval: f64,

    /// command and options
    #[structopt(name = "COMMAND")]
    pub command: Vec<String>,
}
