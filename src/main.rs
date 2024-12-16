
use outbound::dummy::{DummyDecoder, DummyPlayer};
use outbound::cpal_player::CpalPlayer;
use outbound::tokio_loader::TokioReqwestLoader;
use core::symphonia_decoder::SymphoniaDecoder;
use url::Url;
use domain::backend::ports::*;
use domain::frontend::ports::*;

mod domain;
mod inbound;
mod outbound;

mod core;

fn main() {

    env_logger::builder()
    .init();

    log::info!("Hello, world!");


    let fm4_url = Url::parse("https://orf-live.ors-shoutcast.at/fm4-q2a").unwrap();

    let backend = domain::backend::service::new::<TokioReqwestLoader, SymphoniaDecoder, CpalPlayer>();

    backend.send_command(BackendCommand::PlayUrl(fm4_url)).unwrap();
    
    let frontend = inbound::easy_repl::EasyReplFrontend::new();

    frontend.run(backend);
}
