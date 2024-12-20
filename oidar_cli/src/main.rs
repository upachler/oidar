
use oidar_base::outbound::cpal_player::CpalPlayer;
use oidar_base::outbound::tokio_loader::TokioReqwestLoader;
use oidar_base::core::symphonia_decoder::SymphoniaDecoder;
use url::Url;
use oidar_base::domain::backend::ports::*;
use oidar_base::domain::frontend::ports::*;

mod inbound;

fn main() {

    env_logger::builder()
    .init();

    log::info!("Hello, world!");


    let fm4_url = Url::parse("https://orf-live.ors-shoutcast.at/fm4-q2a").unwrap();

    let backend = oidar_base::domain::backend::service::new::<TokioReqwestLoader, SymphoniaDecoder, CpalPlayer>();

    backend.send_command(BackendCommand::PlayUrl(fm4_url)).unwrap();
    
    let frontend = inbound::easy_repl::EasyReplFrontend::new();

    frontend.run(backend);
}
