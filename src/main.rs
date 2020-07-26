#![feature(proc_macro_hygiene, decl_macro)]

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};

//Logging
#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::*;

//Indexing
extern crate tantivy;
mod cli;
mod index;

#[macro_use]
extern crate serde_derive;

//Webserver
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rust_embed;

//Watcher
mod metadata;
mod server;
mod watch;

use index::document_repository::local_repository::LocalDocumentRepository;
use index::file_repository::local_repository::LocalFileRepository;
use index::JobType;

//Error Handling
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ShreddrError {
    #[error("cannot create index")]
    IndexCreation(#[from] index::IndexError),
    #[error("mandatory parameter `{0}` is missing")]
    ParameterMissing(String),
    #[error("directory `{0}` does not exist")]
    DirectoryDoesNotExist(PathBuf),
    #[error("could not load/create configuration")]
    ConfigError(#[from] cli::CLIError),
    #[error("cannot spawn threads")]
    ThreadingError(#[from] std::io::Error),
    #[error("cannot join threads")]
    JoinThreadError(),
    #[error("could not get lock for index")]
    LockError(),
    #[error("could not set log")]
    LogError(#[from] log::SetLoggerError),
    #[error("Could not initialize file repository: {0}")]
    FileRepoError(#[from] index::file_repository::local_repository::LocalFileRepositoryError),
    #[error("Could not initialize document repository: {0}")]
    DocRepoError(#[from] index::document_repository::local_repository::IndexerError),
    #[error("Requires at least one tesseract language to function")]
    NoTesseractLanguagesError(),
}

fn main() -> Result<(), ShreddrError> {
    let cfg = cli::init_cli()?;

    if !cfg.data_dir.exists() {
        println!("DATA directory does not exist");
        return Err(ShreddrError::DirectoryDoesNotExist("DATA".into()));
    }

    if !cfg.consume_dir.exists() {
        println!("CONSUME directory does not exist");
        return Err(ShreddrError::DirectoryDoesNotExist("CONSUME".into()));
    }

    // Init local file repository
    let doc_dir = cfg.data_dir.join("documents");
    std::fs::create_dir_all(&doc_dir).unwrap();
    let file_repo = match LocalFileRepository::new(&doc_dir) {
        Ok(r) => r,
        Err(e) => {
            println!("Could not initialize file repository: {}", &e);
            return Err(e.into());
        }
    };

    // Init local document repository
    let index_dir = cfg.data_dir.join("index");
    std::fs::create_dir_all(&index_dir).unwrap();
    let doc_repo = match LocalDocumentRepository::new(&index_dir) {
        Ok(r) => r,
        Err(e) => {
            println!("Could not initialize file repository: {}", &e);
            return Err(e.into());
        }
    };

    if cfg.tesseract_languages.is_empty() {
        println!("Requires at least one tesseract language to function");
        return Err(ShreddrError::NoTesseractLanguagesError());
    }

    // Init backend
    let index = Arc::new(
        match index::Index::new(
            &cfg.data_dir,
            &cfg.tesseract_languages,
            Arc::new(RwLock::new(file_repo)),
            Arc::new(RwLock::new(doc_repo)),
        ) {
            Ok(i) => i,
            Err(e) => {
                println!("Could not create document index: {}", e);
                return Err(e.into());
            }
        },
    );

    //Start watcher thread
    let (doc_sender, doc_retriever) = crossbeam_channel::unbounded::<JobType>();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let consume_dir = cfg.consume_dir.clone();
    let watch_sender = doc_sender.clone();
    let _w = rt.spawn(async move {
        let watch = watch::PDFWatcher::new(&consume_dir);
        watch.watch(watch_sender).await
    });

    //Start indexer thread
    let i = index.clone();
    let _p = rt.spawn(async move {
        loop {
            let job = doc_retriever.recv().unwrap();
            i.handle_job(job).unwrap();
        }
    });

    // Init logging file
    let log_file_path = cfg.data_dir.join("shreddr.log");
    let log_file = match std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .append(true)
        .open(&log_file_path)
    {
        Err(e) => {
            println!("Could not open log file `{:#?}`", log_file_path);
            return Err(e.into());
        }
        Ok(f) => f,
    };

    //Start server or CLI
    if cfg.server {
        let mut loggers: Vec<Box<dyn SharedLogger>> = vec![];
        //Initialize terminal logging
        match TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed) {
            Some(t) => loggers.push(t),
            None => {
                println!("Started in non-interactive terminal, falling back to write logger");
                loggers.push(WriteLogger::new(
                    LevelFilter::Info,
                    Config::default(),
                    std::io::stdout(),
                ));
            }
        };
        //Initialize logfile logging
        loggers.push(WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            log_file,
        ));
        match CombinedLogger::init(loggers) {
            Ok(_) => {}
            Err(e) => println!("Could not initialize logger {}", e),
        };
        //Start Server
        server::Server::start(&cfg.data_dir, index, Mutex::new(doc_sender));
    } else {
        match WriteLogger::init(LevelFilter::Debug, Config::default(), log_file) {
            Ok(_) => {}
            Err(e) => println!("Could not initialize logger {}", e),
        };
        cli::run_shell(index);
    }

    rt.shutdown_timeout(std::time::Duration::from_millis(100));
    Ok(())
}
