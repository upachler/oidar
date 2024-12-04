use backend::Backend;
use frontend::Frontend;
use url::Url;

mod frontend;
mod backend;

mod dummy;

fn main() {

    println!("Hello, world!");


    let fm4_url = Url::parse("https://orf-live.ors-shoutcast.at/fm4-q2a").unwrap();

    let loader = dummy::DummyLoader::new();
    let decoder = dummy::DummyDecoder::new();
    let player = dummy::DummyPlayer::new();

    let backend = backend::new(loader, decoder, player);

    backend.send_command(backend::BackendCommand::PlayUrl(fm4_url));
    
    let frontend = frontend::easy_repl::EasyReplFrontend::new();

    frontend.run(backend);
}
