use clap::clap_app;
use std::sync::Arc;
extern crate confy;
extern crate serde_derive;
use std::path::PathBuf;

use shrust::{Shell, ShellIO};
use std::io::prelude::*;

/// All available Shreddr configuration values
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShreddrConfig {
    #[serde(default)]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub consume_dir: PathBuf,
    #[serde(default)]
    pub server: bool,
    /// Languages to use with tesseract. Each language file has to be installed in the system.
    /// The order of languages is used as priority for tesseract.
    #[serde(default)]
    pub tesseract_languages: Vec<String>,
    #[serde(default)]
    pub max_upload_size: u64,
    #[serde(default)]
    pub extract_extended_metadata: bool,
}

impl Default for ShreddrConfig {
    fn default() -> Self {
        ShreddrConfig {
            data_dir: PathBuf::new(),
            consume_dir: PathBuf::new(),
            server: false,
            tesseract_languages: vec![],
            max_upload_size: 20 * 1024 * 1024,
            extract_extended_metadata: true,
        }
    }
}

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum CLIError {
    #[error("could not load configuration file")]
    LoadConfigError(#[from] confy::ConfyError),
}

/// Initializes the CLI with command line arguments and loads the configuration from the config file.
pub fn init_cli() -> Result<ShreddrConfig, CLIError> {
    let args = clap_app!(shreddr =>
        (about: "A document management system")
        (version: clap::crate_version!())
        (author: clap::crate_authors!())
        (@arg DATA: -d --data_dir +takes_value "Sets the storage and config directory")
        (@arg CONSUME: -c --consume +takes_value "Sets the directory from which PDF files are consumed")
        (@arg CONFIG: --config +takes_value "Sets the config file to use")
        (@arg TESSERACT_LANG: -t --tesseract_lang +takes_value +multiple "Which tesseract languages to use (ISO 639)")
        (@arg SERVER: -s --server "Starts the shreddr server")
    ).get_matches();
    load_config(&args)
}

/// Loads a configuration file if it exists,
/// and optionally overrides options passed as command line arguments
fn load_config(cli_arguments: &clap::ArgMatches) -> Result<ShreddrConfig, CLIError> {
    let mut cfg: ShreddrConfig;

    //Load config file
    if let Some(c) = cli_arguments.value_of("CONFIG") {
        info!("Loading config from file '{}'", c);
        cfg = confy::load_path(c)?;
    } else {
        cfg = confy::load("shreddr")?;
    }

    //Check if CLI overwrites
    if let Some(d) = cli_arguments.value_of("DATA") {
        cfg.data_dir = d.into();
    };
    if let Some(d) = cli_arguments.value_of("CONSUME") {
        cfg.consume_dir = d.into();
    };
    if cli_arguments.occurrences_of("SERVER") > 0 {
        cfg.server = true;
    }
    if let Some(t) = cli_arguments.values_of("TESSERACT_LANG") {
        cfg.tesseract_languages = t.map(|s| s.into()).collect();
    };

    Ok(cfg)
}

/// Executes the Shell I/O
pub fn run_shell(index: Arc<super::index::Index>) {
    print!("{}", header());
    let mut shell = Shell::new(index);

    shell.new_command("get", "Retrieves a document", 1, |io, index, s| {
        let id = s[0].parse::<u64>();
        match id {
            Ok(i) => {
                let doc = index.get_document_path(i);
                match doc {
                    Ok(path) => writeln!(io, "Document path:{:#?}", path)?,
                    Err(e) => writeln!(io, "Error during reprocess: {}", e)?,
                };
            }
            Err(e) => {
                writeln!(io, "Could not parse number {}", e)?;
            }
        }
        Ok(())
    });

    shell.new_command("reprocess", "Reprocesses a document", 1, |io, index, s| {
        let id = s[0].parse::<u64>();
        match id {
            Ok(i) => match index.reprocess_document(i) {
                Ok(()) => writeln!(io, "Reprocessed document {}", i)?,
                Err(e) => writeln!(io, "Error during reprocess: {}", e)?,
            },
            Err(e) => writeln!(io, "Could not parse number {}", e)?,
        }
        Ok(())
    });

    shell.new_command(
        "remove",
        "Removes a document from the index",
        1,
        |io, index, s| {
            let id = s[0].parse::<u64>();
            match id {
                Ok(i) => match index.remove_document(i) {
                    Ok(()) => writeln!(io, "Removed document {}", i)?,
                    Err(e) => writeln!(io, "Error during removal: {}", e)?,
                },
                Err(e) => writeln!(io, "Could not parse number {}", e)?,
            }
            Ok(())
        },
    );

    shell.new_command("import", "Imports a document", 1, |io, index, s| {
        let p: PathBuf = s[0].into();
        if !(p.exists() && p.is_file()) {
            writeln!(io, "Document at path {:#?} does not exist.", p)?;
            return Ok(());
        }
        match index.import_document(&p, true) {
            Ok(id) => writeln!(io, "Imported document {}", id)?,
            Err(e) => writeln!(io, "Error during reprocess: {}", e)?,
        }

        Ok(())
    });

    shell.new_command(
        "addtag",
        "Adds a tag with the given name",
        1,
        |io, index, s| {
            let selections = &["Full Match", "Regex"];
            let selection =
                dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("What type of matcher should the tag use?")
                    .default(0)
                    .items(&selections[..])
                    .interact()?;
            let tag = match selection {
                0 => {
                    //Fullmatch
                    let match_str: String =
                        dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                            .with_prompt("Match String")
                            .interact()?;
                    let case_insensitive =
                        dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                            .with_prompt("Case insensitive?")
                            .interact()?;
                    super::metadata::tag::TagConfig {
                        id: 0,
                        name: s[0].into(),
                        color: None,
                        matcher: super::metadata::tag::MatcherConfig::FullMatcher {
                            match_str,
                            case_insensitive,
                        },
                    }
                }
                1 => {
                    //Regex Match
                    let match_str: String =
                        dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                            .with_prompt("Match String")
                            .interact()?;
                    super::metadata::tag::TagConfig {
                        id: 0,
                        name: s[0].into(),
                        color: None,
                        matcher: super::metadata::tag::MatcherConfig::RegexMatcher { match_str },
                    }
                }
                x => {
                    writeln!(io, "Invalid selection: {}", x)?;
                    return Ok(());
                }
            };
            index.add_tag(tag)?;
            Ok(())
        },
    );
    shell.run_loop(&mut ShellIO::default());
}

fn header() -> String {
    r#"

     _____ _                  _     _      
    /  ___| |                | |   | |     
    \ `--.| |__  _ __ ___  __| | __| |_ __ 
     `--. \ '_ \| '__/ _ \/ _` |/ _` | '__|
    /\__/ / | | | | |  __/ (_| | (_| | |   
    \____/|_| |_|_|  \___|\__,_|\__,_|_|   
                                                
    "#
    .into()
}
