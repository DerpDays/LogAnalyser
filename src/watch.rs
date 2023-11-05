use notify::event::RemoveKind;
use notify::{
    event::{CreateKind, ModifyKind},
    Watcher,
};
use notify::{Event, EventKind};
use std::path::PathBuf;
use tokio::fs::{File, ReadDir};
use tokio::io::AsyncSeekExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};


// example:
// #[tokio::main]
// async fn main() -> () {
//     let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//
//     tokio::task::spawn(async move { watch_logs(&PathBuf::from("./logs"), tx).await });
//     let _ = process_logs(rx).await;
//
//     ()
// }


struct FileHandle {
    path: PathBuf,
    handle: File,
}

pub struct RawLogData {
    path: PathBuf,
    lines: Vec<String>,
}


async fn files_from_dir(mut directory: ReadDir) -> Vec<FileHandle> {
    let mut files: Vec<FileHandle> = vec![];
    while let Ok(file) = directory.next_entry().await {
        match file {
            Some(dir_entry) => {
                if let Ok(file) = File::open(dir_entry.path()).await {
                    files.push(FileHandle {
                        path: dir_entry.path(),
                        handle: file,
                    });
                }
            }
            None => break,
        };
    }
    files
}

async fn process_logs(mut rx: UnboundedReceiver<Vec<RawLogData>>) -> () {
    while let Some(log) = rx.recv().await {
        for data in log.iter() {
            println!("{:?}", data.lines);
        }
    }
}

pub async fn watch_logs(path: &PathBuf, out: UnboundedSender<Vec<RawLogData>>) -> () {
    // Create a MPSC channel used to recieve any updates to the log directory.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |event: Result<notify::Event, _>| {
        if let Ok(event) = event {
            let _ = tx.send(event);
        }
    })
    .unwrap();

    // Watch the logs directory for any changes.
    watcher
        .watch(&path, notify::RecursiveMode::Recursive)
        .expect("Could not watch the log directory");

    // Add the open all of the files within the logs directory.
    let directory: ReadDir = tokio::fs::read_dir(path)
        .await
        .expect("Could not read the log directory");
    let mut files: Vec<FileHandle> = files_from_dir(directory).await;
    for handle in files.iter_mut() {
        if let Some(lines) = read_till_end(&mut handle.handle).await {
            let _ = out.send(vec![RawLogData {
                path: path.to_owned(),
                lines,
            }]);
        }
    }

    on_file_event(&mut rx, &mut files, out).await;
}

async fn on_file_event(
    from: &mut UnboundedReceiver<notify::Event>,
    files: &mut Vec<FileHandle>,
    to: UnboundedSender<Vec<RawLogData>>,
) -> () {
    // On recieving an update form the watcher.
    while let Some(event) = from.recv().await {
        match event.kind {
            // If the file has been modified.
            EventKind::Modify(ModifyKind::Data(_)) => {
                let mut new_lines: Vec<RawLogData> = vec![];
                for handle in files.iter_mut() {
                    if event.paths.iter().any(|p| {
                        PathBuf::from(p).canonicalize().unwrap()
                            == handle.path.canonicalize().unwrap()
                    }) {
                        if let Some(lines) = read_till_end(&mut handle.handle).await {
                            new_lines.push(RawLogData {
                                path: handle.path.to_path_buf(),
                                lines,
                            });
                        }
                    }
                }
                let _ = to.send(new_lines);
            }
            // If the a new file has been created.
            EventKind::Create(CreateKind::File) => {
                for i in &event.paths {
                    if let Ok(mut file) = File::open(i).await {
                        let path = i.to_path_buf();
                        if let Some(lines) = read_till_end(&mut file).await {
                            let _ = to.send(vec![RawLogData {
                                path: path.clone(),
                                lines,
                            }]);
                        }
                        files.push(FileHandle { path, handle: file });
                    }
                }
            }
            EventKind::Remove(RemoveKind::Any) => {
                for i in &event.paths {
                    if let Some(pos) = files
                        .iter()
                        .position(|handle| &handle.path == &i.to_path_buf())
                    {
                        files.remove(pos);
                    }
                }
            }
            _ => {}
        }
        println!("recv: {event:?}")
    }
}

async fn read_till_end(file: &mut File) -> Option<Vec<String>> {
    // ignore any event that didn't change the pos
    let pos = file.stream_position().await.unwrap();
    let len = file.metadata().await.unwrap().len();
    if len <= pos {
        return None;
    }

    // read 1 byte onwards from the current seek position.
    let _ = file.seek(tokio::io::SeekFrom::Current(1));

    let mut res: Vec<String> = vec![];
    let mut lines = BufReader::new(file).lines();
    while let Ok(line) = lines.next_line().await {
        match line {
            Some(line) => res.push(line),
            _ => break,
        }
    }
    Some(res)
}





struct Log;

fn adsadsa(inp: Vec<String>) -> Vec<Log> {
    let iter = inp.into_iter();
    inp.into_iter().take_while(|val| {
        val1 = val.split_whitespace();
        if val1.len() >=2 {
            match parsing_date(val1[0:1]) {
                Ok(_) => true,
                Err(_) => false
            }
        } else {
            false
        }
    });
    vec![]
}




// TODO: REFACTOR

// struct DirectoryReadStream {
//     watch_directory: PathBuf,
//     file_handles: Vec<FileHandle>,
//     internal_rx: UnboundedReceiver<Event>,
//     internal_tx: UnboundedSender<Event>,
//     recipient: UnboundedSender<Vec<RawLogData>>,
// }
//
// impl DirectoryReadStream {
//     pub fn new(self, watch_directory: PathBuf, recipient: UnboundedSender<Vec<RawLogData>>) -> Self {
//         // Creates a MPSC channel used to recieve any updates/changes to the files in the watch directory.
//         let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
//         DirectoryReadStream {
//             watch_directory,
//             file_handles: vec![],
//             internal_rx: rx,
//             internal_tx: tx,
//             recipient,
//         }
//     }
//
//     fn send_watch_event<E>(self, event: Result<notify::Event, E>) {
//         if let Ok(event) = event {
//             self.internal_tx.send(event);
//         }
//     }
//     /// Starts watching the files within the watch_directory for any additions
//     /// and sends them to the reciever mpsc.
//     pub async fn start_watching(mut self) -> () {
//         let mut watcher = notify::recommended_watcher(move |event: Result<notify::Event, _>| {
//             if let Ok(event) = event {
//                 let _ = self.internal_tx.send(event);
//             }
//         }).unwrap();
//
//         // Watch the logs directory for any changes.
//         watcher
//             .watch(&self.watch_directory, notify::RecursiveMode::Recursive)
//             .expect("Could not watch the log directory, does it exist and is accessible?");
//
//         // Add the open all of the files within the logs directory.
//         let directory: ReadDir = tokio::fs::read_dir(&self.watch_directory)
//             .await
//             .expect("Could not read the watch directory, doess it exist and is it accessible?.");
//
//         self.file_handles = files_from_dir(directory).await;
//
//         // Read all of the log files initially present and send them to the recipient.
//         for handle in self.file_handles.iter_mut() {
//             if let Some(lines) = read_till_end(&mut handle.handle).await {
//                 let _ = self.recipient.send(vec![RawLogData {
//                     path: handle.path.clone(),
//                     lines,
//                 }]);
//             }
//         }
//
//         // on_file_event(&mut rx, &mut files, out).await;
//         self.on_watch_event().await;
//     }
//
//     async fn on_watch_event(mut self) -> () {
//         // On recieving an update form the watcher.
//         while let Some(event) = &self.internal_rx.recv().await {
//             match event.kind {
//                 // If the file has been modified.
//                 EventKind::Modify(ModifyKind::Data(_)) => {
//                     let mut new_lines: Vec<RawLogData> = vec![];
//                     for handle in self.file_handles.iter_mut() {
//                         if event.paths.iter().any(|p| {
//                             PathBuf::from(p).canonicalize().unwrap()
//                                 == handle.path.canonicalize().unwrap()
//                         }) {
//                             if let Some(lines) = read_till_end(&mut handle.handle).await {
//                                 new_lines.push(RawLogData {
//                                     path: handle.path.to_path_buf(),
//                                     lines,
//                                 });
//                             }
//                         }
//                     }
//                     let _ = self.recipient.send(new_lines);
//                 }
//                 // If the a new file has been created.
//                 EventKind::Create(CreateKind::File) => {
//                     for i in &event.paths {
//                         if let Ok(mut file) = File::open(i).await {
//                             let path = i.to_path_buf();
//                             if let Some(lines) = read_till_end(&mut file).await {
//                                 let _ = self.recipient.send(vec![RawLogData {
//                                     path: path.clone(),
//                                     lines,
//                                 }]);
//                             }
//                             self.file_handles.push(FileHandle { path, handle: file });
//                         }
//                     }
//                 }
//                 EventKind::Remove(RemoveKind::Any) => {
//                     for i in &event.paths {
//                         if let Some(pos) = self.file_handles
//                             .iter()
//                             .position(|handle| &handle.path == &i.to_path_buf())
//                         {
//                             self.file_handles.remove(pos);
//                         }
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }
// }

