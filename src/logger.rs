use fern;
use log;
use time;
use std;
use error::BResult;
use crate::config;

/// a simple wrapper (pretty much direct from documentation) that sets up
/// logging to STDOUT (and file if config allows) via fern/log
pub fn setup_logger() -> BResult<()> {
    let levelstr: String = config::get("logging.level")?;
    let level = match levelstr.to_lowercase().as_ref() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        "off" => log::LevelFilter::Off,
        _ => {
            println!("logger::setup_logger() -- bad `log.level` value (\"{}\"), defaulting to \"warn\"", levelstr);
            log::LevelFilter::Warn
        }
    };
    let config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} - [{}][{}] {}",
                time::now().strftime("%Y-%m-%dT%H:%M:%S").expect("turtl::logger::setup_logger() -- failed to parse time or something"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .level_for("exonum::node::consensus", log::LevelFilter::Warn)
        .level_for("actix_web::pipeline", log::LevelFilter::Error)
        .chain(std::io::stdout());
    match config.apply() {
        Ok(_) => {}
        Err(e) => {
            trace!("logger::setup_logger() -- looks like the logger was already init: {}", e);
        }
    }
    Ok(())
}

