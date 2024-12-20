use easy_repl::{command, Repl, CommandStatus};
use oidar_base::domain::backend::ports::Backend;
use std::rc::Rc;
use oidar_base::domain::backend::ports::*;
use url::Url;
use oidar_base::domain::frontend::ports::Frontend;

pub struct EasyReplFrontend {
}

impl EasyReplFrontend {
    pub fn new() -> Self{
        Self {}
    }
}

impl Frontend for EasyReplFrontend {
    fn run(&self, backend: impl Backend) {
        let backend = Rc::new(backend);
        let backend_play = backend.clone();
        let backend_stop = backend.clone();
        let mut repl = Repl::builder()
        .add("play", command! {
            "play URL",
            (url: String) => |url: String| {
                match Url::parse(&url) {
                    Ok(url) => backend_play.send_command(BackendCommand::PlayUrl(url)).unwrap(),
                    Err(e) => eprint!("invalid url '{url}' provided, error: {e}"),
                };
                Ok(CommandStatus::Done)
            }
        })
        .add("stop", command!{
            "stop playback",
            () => || {
                backend_stop.send_command(BackendCommand::StopPlayback).unwrap();
                Ok(CommandStatus::Done)
            }
        })
        .add("exit", command! {
            "exit player",
            () => || {
                Ok(CommandStatus::Quit)
            }
        })
        .build().expect("Failed to create repl");

        repl.run().unwrap();
    }

}