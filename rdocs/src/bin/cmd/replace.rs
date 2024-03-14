use std::path::{Path, PathBuf};

use rdocs::{
    cli::CmdExit,
    collect, parser,
    replacer::{self, ReplaceStatus},
};
use serde::{Deserialize, Serialize};
use tabled::{builder::Builder, settings::Style};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    parser: parser::Config,
    collector: collect::Config,
}

pub fn exec(
    config_path: Option<&PathBuf>,
    collect_folder: &Path,
    replace_folder: &Path,
    dry_run: bool,
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
    let parser_result = parser.extract_content(&collector);

    let collector = match collect::Collector::from_config(replace_folder, &config.collector) {
        Ok(collector) => collector,
        Err(err) => {
            return CmdExit::error_with_message(&format!("could not init collector: {err}"));
        }
    };

    let replace_results = {
        let mut replace_results = if dry_run {
            replacer::Replace::default().stats(&collector, &parser_result)
        } else {
            replacer::Replace::default().replace_content(&collector, &parser_result)
        };
        replace_results.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
        replace_results
    };

    let mut builder = Builder::default();
    builder.push_record(["id", "status", "path"]);

    for result in &replace_results {
        let (id, content) = match &result.status {
            ReplaceStatus::NotFound(_) | ReplaceStatus::Error(_) => continue,
            ReplaceStatus::Equal(id) => (id.to_string(), String::new()),
            ReplaceStatus::Replaced(id, _, block) => (id.to_string(), block.to_string()),
        };

        builder.push_record([
            id,
            result.status.to_string(),
            result.path.display().to_string(),
            content,
        ]);
    }

    if builder.count_records() > 1 {
        if std::env::var("TEST").is_ok() {
            let res: Vec<Vec<String>> = builder.into();
            println!("{res:#?}");
        } else {
            let table = builder.build().with(Style::modern()).to_string();
            println!("{table}");
        }
    } else {
        return CmdExit::error_with_message("Not found block to replace");
    }

    let has_error = replace_results
        .iter()
        .filter(|&r| matches!(r.status, ReplaceStatus::Error(_)))
        .count();

    if has_error > 0 {
        CmdExit::error_with_message("Finished with errors")
    } else {
        CmdExit::ok()
    }
}
