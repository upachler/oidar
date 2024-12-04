use easy_repl::{command, Repl, CommandStatus};
use std::rc::Rc;
use crate::domain::backend::ports::*;
use url::Url;
use crate::domain::frontend::ports::Frontend;

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
        let backend_clone = backend.clone();
        let mut repl = Repl::builder()
        .add("play", command! {
            "play URL",
            (url: String) => |url: String| {
                match Url::parse(&url) {
                    Ok(url) => backend_clone.send_command(BackendCommand::PlayUrl(url)),
                    Err(e) => eprint!("invalid url '{url}' provided, error: {e}"),
                };
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