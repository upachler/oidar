
use url::Url;
use domain::backend::ports::*;
use domain::frontend::ports::*;

mod domain;
mod dummy;

mod easy_repl;

fn main() {

    println!("Hello, world!");


    let fm4_url = Url::parse("https://orf-live.ors-shoutcast.at/fm4-q2a").unwrap();

    let loader = dummy::DummyLoader::new();
    let decoder = dummy::DummyDecoder::new();
    let player = dummy::DummyPlayer::new();

    let backend = domain::backend::service::new(loader, decoder, player);

    backend.send_command(BackendCommand::PlayUrl(fm4_url));
    
    let frontend = easy_repl::EasyReplFrontend::new();

    frontend.run(backend);
}
