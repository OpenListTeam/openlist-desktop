use log::LevelFilter;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

pub fn init_log() -> Result<(), Box<dyn std::error::Error>> {
    let trigger = SizeTrigger::new(10 * 1024 * 1024); // 10 MB
    let log_file_dir = super::path::get_app_logs_dir().map_err(|e| e.to_string())?;
    let log_file_path = log_file_dir.join("app.log");
    let archive_pattern = log_file_dir.join("compressed-log.{}.log");
    let roller = FixedWindowRoller::builder()
        .build(archive_pattern.into_os_string().to_str().unwrap(), 3)
        .unwrap();
    let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));
    let logfile = log4rs::append::rolling_file::RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S.%f)} [{t}] {l:5} {m}\n",
        )))
        .build(log_file_path, Box::new(policy))
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config)?;
    Ok(())
}
