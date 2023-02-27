//! Logging for the library.
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};

const SCRAPER_LOGFILE: &str = "./log/saptest_scraper.log";
const DB_LOGFILE: &str = "./log/saptest_db.log";
const RUN_LOGFILE: &str = "./log/saptest_run.log";

/// Builds library [`log4rs`](https://docs.rs/log4rs/latest/log4rs/) configuration.
///
/// Logfiles:
/// 1. `./log/saptest_scraper.log`
///     * Pet, token, and food information scraped and errors.
/// 2. `./log/saptest_db.log`
///     * Database entries created and errors.
/// 3. `./log/saptest_run.log`
///     * Run log showing breakdown of steps.
///
/// ```rust no_run
/// use saptest::logging::build_log_config;
///
/// fn main() {
///     let config = build_log_config();
///     log4rs::init_config(config).unwrap();
///
///     // Code here.
/// }
/// ```
pub fn build_log_config() -> Config {
    // https://github.com/estk/log4rs/blob/master/examples/log_to_file.rs
    let pattern = "{d} {l} {M} - {m}{n}";
    let db_logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(DB_LOGFILE)
        .unwrap();
    let scraper_logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(SCRAPER_LOGFILE)
        .unwrap();
    let run_logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(RUN_LOGFILE)
        .unwrap();

    Config::builder()
        .appender(Appender::builder().build("database", Box::new(db_logfile)))
        .appender(Appender::builder().build("scraper", Box::new(scraper_logfile)))
        .appender(Appender::builder().build("run", Box::new(run_logfile)))
        .logger(
            Logger::builder()
                .appender("database")
                .additive(false)
                .build("db", LevelFilter::Debug),
        )
        .logger(
            Logger::builder()
                .appender("scraper")
                .additive(false)
                .build("wiki_scraper", LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .appender("run")
                .additive(false)
                .build("run", LevelFilter::Info),
        )
        .build(Root::builder().build(LevelFilter::Info))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{build_log_config, DB_LOGFILE, RUN_LOGFILE, SCRAPER_LOGFILE};

    #[test]
    fn test_log_files() {
        let db_log_file = Path::new(DB_LOGFILE);
        let scraper_log_file = Path::new(SCRAPER_LOGFILE);
        let run_log_file = Path::new(RUN_LOGFILE);

        build_log_config();

        assert!(
            db_log_file.try_exists().is_ok()
                && scraper_log_file.try_exists().is_ok()
                && run_log_file.try_exists().is_ok()
        );
    }
}
