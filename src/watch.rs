use std::path::{Path, PathBuf};

//Notify
extern crate notify;
use notify::DebouncedEvent::Create;
use notify::Watcher;

use crate::index::JobType;

use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

pub struct PDFWatcher {
    dir: PathBuf,
}

impl PDFWatcher {
    pub fn new(dir: &Path) -> PDFWatcher {
        PDFWatcher { dir: dir.into() }
    }

    /// Checks if the given file matches.
    /// Currently, the watcher only checks for the PDF extention
    /// ```
    /// let file: PathBuf = "/tmp/file.pdf";
    /// assert_true(PDFWatcher::match_file(&file));
    /// ```
    pub fn match_file(file: &Path) -> bool {
        match file.extension() {
            None => false,
            Some(os_str) => match os_str.to_str() {
                Some("pdf") => true,
                _ => false,
            },
        }
    }

    pub async fn watch(
        &self,
        sender: Sender<JobType>,
    ) -> Result<(), Box<dyn std::error::Error + Send>> {
        self.init_dir(&sender);
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.
        let mut watcher = notify::watcher(tx, Duration::from_secs(10)).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(&self.dir, notify::RecursiveMode::Recursive)
            .unwrap();
        info!("Watching directory {:#?}", &self.dir);

        loop {
            match rx.recv() {
                Ok(event) => match event {
                    Create(f) => {
                        if PDFWatcher::match_file(&f) {
                            info!("PDF-file created in watched dir: {:?}", f);
                            sender.send(JobType::ImportFile{path:f}).unwrap();
                        } else {
                            debug!("Ignored file: {:?}", f)
                        }
                    }
                    x => debug!("Ignored event: {:?}", x),
                },
                Err(e) => {
                    error!("Could not watch directory: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        }
    }

    fn init_dir(&self, sender: &Sender<JobType>) {
        let paths = std::fs::read_dir(&self.dir).unwrap();
        for path in paths {
            let p = path.unwrap().path();
            sender.send(JobType::ImportFile{path:p}).unwrap();
        }
    }
}
