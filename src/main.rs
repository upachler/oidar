
use url::Url;
use domain::backend::ports::*;
use domain::frontend::ports::*;

mod domain;
mod inbound;
mod outbound;



fn main() {

    env_logger::builder()
    .init();

    log::info!("Hello, world!");


    let fm4_url = Url::parse("https://orf-live.ors-shoutcast.at/fm4-q2a").unwrap();

    //let loader = outbound::dummy::DummyLoader::new();
    let loader = outbound::tokio_loader::TokioReqwestLoader::new();
    let decoder = outbound::dummy::DummyDecoder::new();
    let player = outbound::dummy::DummyPlayer::new();

    let backend = domain::backend::service::new(loader, decoder, player);

    backend.send_command(BackendCommand::PlayUrl(fm4_url)).unwrap();
    
    let frontend = inbound::easy_repl::EasyReplFrontend::new();

    frontend.run(backend);
}
