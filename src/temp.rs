use notify::EventKind;
use notify::{
    event::{CreateKind, ModifyKind},
    Watcher,
};
use std::path::PathBuf;
use tokio::fs::{File, ReadDir};
use tokio::io::AsyncSeekExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[tokio::main]
async fn main() -> () {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::task::spawn(async move { watch_logs(&PathBuf::from("./logs"), tx).await });
    let _ = process_logs(rx).await;

    ()
}

async fn open_dir(mut directory: ReadDir) -> Vec<(PathBuf, File)> {
    let mut files: Vec<(PathBuf, File)> = vec![];
    while let Ok(file) = directory.next_entry().await {
        match file {
            Some(dir_entry) => {
                if let Ok(file) = File::open(dir_entry.path()).await {
                    files.push((dir_entry.path(), file));
                }
            }
            None => break,
        };
    }
    files
}

async fn process_logs(mut rx: UnboundedReceiver<Vec<(PathBuf, Vec<String>)>>) -> () {
    while let Some(change) = rx.recv().await {
        for i in change.iter() {
            println!("{:?}", i);
        }
    }
}

async fn watch_logs(path: &PathBuf, out: UnboundedSender<Vec<(PathBuf, Vec<String>)>>) -> () {
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
    let mut files: Vec<(PathBuf, File)> = open_dir(directory).await;
    for (path, file) in files.iter_mut() {
        let _ = out.send(vec![(path.to_path_buf(), read_till_end(file).await)]);
    }

    on_modify(&mut rx, &mut files, out).await;
}

async fn on_modify(
    from: &mut UnboundedReceiver<notify::Event>,
    files: &mut Vec<(PathBuf, File)>,
    to: UnboundedSender<Vec<(PathBuf, Vec<String>)>>,
) -> () {
    // On recieving an update form the watcher.
    while let Some(event) = from.recv().await {
        match event.kind {
            // If the file has been modified.
            EventKind::Modify(ModifyKind::Data(_)) => {
                let mut new_lines: Vec<(PathBuf, Vec<String>)> = vec![];
                for (path, file) in files.iter_mut() {
                    if event.paths.iter().any(|p| {
                        PathBuf::from(p).canonicalize().unwrap() == path.canonicalize().unwrap()
                    }) {
                        let content = read_till_end(file).await;
                        new_lines.push((path.to_path_buf(), content));
                    }
                }
                let _ = to.send(new_lines);
            }
            // If the a new file has been created.
            EventKind::Create(CreateKind::Any) => {
                for i in &event.paths {
                    if let Ok(mut file) = File::open(i).await {
                        let path = i.to_path_buf();
                        println!("");
                        
                        let _ = to.send(vec![(path.clone(), read_till_end(&mut file).await)]);
                        files.push((path.clone(), file));
                    }
                }
            }
            _ => {}
        }
        println!("recv: {event:?}")
    }
}

async fn read_till_end(file: &mut File) -> Vec<String> {
    // ignore any event that didn't change the pos
    let pos = file.stream_position().await.unwrap();
    let len = file.metadata().await.unwrap().len();
    if len <= pos {
        return vec![];
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
    res
}
