use std::path::{Path, PathBuf};

use rdocs::{cli::CmdExit, collect, out, parser};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    parser: parser::Config,
    collector: collect::Config,
}

pub fn exec(
    config_path: Option<&PathBuf>,
    collect_folder: &Path,
    format: Option<out::Format>,
    output: Option<PathBuf>,
) -> CmdExit {
    let span = tracing::span!(tracing::Level::TRACE, "exec");
    let _guard = span.enter();

    let config = match config_path {
        Some(path) => {
            let rdr = match std::fs::File::open(path) {
                Ok(rdr) => rdr,
                Err(err) => {
                    return CmdExit::error_with_message(&format!(
                        "could not read config file: {err}"
                    ));
                }
            };

            match serde_yaml::from_reader(rdr) {
                Ok(config) => config,
                Err(err) => {
                    return CmdExit::error_with_message(&format!("invalid config file: {err}"));
                }
            }
        }
        None => Config::default(),
    };

    let collector = match collect::Collector::from_config(collect_folder, &config.collector) {
        Ok(collector) => collector,
        Err(err) => {
            return CmdExit::error_with_message(&format!("could not init collector: {err}"));
        }
    };
    let parser = parser::Parser::with_config(config.parser);

    let results = parser.extract_content(&collector);

    if results.is_empty() {
        CmdExit::error_with_message("code captures not found in the given path")
    } else {
        let out = match format {
            Some(format) => out::Content::All(out::Output::new(output), format),
            None => out::Content::Only(out::Output::new(output)),
        };
        if let Err(err) = out.export(results) {
            CmdExit::error_with_message(&format!("export result error: {err}"))
        } else {
            CmdExit::ok()
        }
    }
}
