use super::ports::*;
use std::{sync::mpsc::*, thread::JoinHandle};


struct BackendCoreStub {
    command_sender: SyncSender<BackendCommand>,
    event_receiver: Receiver<BackendEvent>,

    join_handle: JoinHandle<()>,
}

pub fn new<L,D,P>(loader: L, decoder: D, player: P) -> impl Backend 
where 
    L: Loader + Send + 'static,
    D: Decoder + Send + 'static,
    P: Player + Send + 'static,
{
    let (command_sender, command_receiver) = sync_channel(10);
    let (event_sender, event_receiver) = sync_channel(10);

    let core = BackendCore {
        command_receiver,
        event_sender, 
        decoder, 
        loader, 
        player,
    };

    let join_handle = std::thread::spawn(move ||exec_core_thread(core));

    BackendCoreStub {
        command_sender,
        event_receiver,
        join_handle
    }
}

impl Drop for BackendCoreStub {
    fn drop(&mut self) {
        self.command_sender.send(BackendCommand::Shutdown);
    }
}

impl Backend for BackendCoreStub {
    fn event_receiver(&self) -> &Receiver<BackendEvent> {
        &self.event_receiver
    }
    fn send_command(&self, cmd: BackendCommand) {
        self.command_sender.send(cmd);
    }
}

struct BackendCore<L,D,P> 
{
    command_receiver: Receiver<BackendCommand>,
    event_sender: SyncSender<BackendEvent>,

    loader: L,
    decoder: D,
    player: P,
}

fn exec_core_thread<L,D,P>(core: BackendCore<L,D,P>) 
{

}

