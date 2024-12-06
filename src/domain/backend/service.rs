use super::ports::*;
use std::{sync::mpsc::*, thread::JoinHandle};
use anyhow::Result;
use url::Url;


struct BackendCoreStub {
    command_sender: SyncSender<BackendCommand>,
    event_receiver: Receiver<BackendEvent>,

    join_handle: JoinHandle<()>,
}

enum LoaderCmd {
    SetUrl(Url),
    Stop,
}

pub fn new<L,D,P>(mut loader: L, mut decoder: D, player: P) -> impl Backend 
where 
    L: Loader + Send + 'static,
    D: Decoder + Send + 'static,
    P: Player + Send + 'static,
{
    let (command_sender, command_receiver) = sync_channel(10);
    let (event_sender, event_receiver) = sync_channel(10);
    let (loadercmd_sender, loadercmd_receiver) = sync_channel::<LoaderCmd>(10);

    let (chunk_sender, chunk_receiver) = sync_channel(2);
    std::thread::spawn(move ||{
        let mut running = true;
        loop {
            let cmd = if running {
                match loadercmd_receiver.try_recv() {
                    Ok(cmd) => Some(cmd),
                    Err(TryRecvError::Disconnected) => return,
                    Err(TryRecvError::Empty) => None,
                }
            } else {
                match loadercmd_receiver.recv() {
                    Ok(cmd) => Some(cmd),
                    Err(e)  => return,
                }
            };

            if let Some(cmd) = cmd {
                match cmd {
                    LoaderCmd::SetUrl(url) => loader.set_url(url),
                    LoaderCmd::Stop => {
                        running = false;
                        continue;
                    }
                }
            }
            
            match loader.read_chunk() {
                Ok(chunk) => chunk_sender.send(chunk).unwrap(),
                Err(_) => break,
            }
        }
    });

    let (frame_sender, frame_receiver) = sync_channel(2);
    std::thread::spawn(move ||{
        loop {
            match decoder.decode() {
                DecoderState::NeedChunk => {
                    let chunk = match chunk_receiver.recv(){
                        Ok(chunk) => chunk,
                        Err(_) => return,
                    };
                    decoder.push_chunk(chunk);
                }
                DecoderState::FinishedFrame(frame) => frame_sender.send(frame).unwrap(),
            }
        }
    });

    std::thread::spawn(move ||{
        loop {
            match frame_receiver.recv() {
                Ok(frame) => player.play(frame),
                Err(_) => return
            }
        }
    });

    let join_handle = std::thread::spawn(move ||exec_core_thread(command_receiver, loadercmd_sender));

    BackendCoreStub {
        command_sender,
        event_receiver,
        join_handle
    }
}

impl Drop for BackendCoreStub {
    fn drop(&mut self) {
        let _ = self.command_sender.send(BackendCommand::Shutdown);
    }
}

impl Backend for BackendCoreStub {
    fn event_receiver(&self) -> &Receiver<BackendEvent> {
        &self.event_receiver
    }
    fn send_command(&self, cmd: BackendCommand) -> Result<()> {
        Ok(self.command_sender.send(cmd)?)
    }
}

struct BackendCore
{
    command_receiver: Receiver<BackendCommand>,
    event_sender: SyncSender<BackendEvent>,
}

fn exec_core_thread(command_receiver: Receiver<BackendCommand>, loadercmd_sender: SyncSender<LoaderCmd>) 
{

    loop {
        let cmd = match command_receiver.recv() {
            Ok(cmd) => cmd,
            Err(_) => return,
        };

        match cmd {
            BackendCommand::PlayUrl(url) => loadercmd_sender.send(LoaderCmd::SetUrl(url)).unwrap(),
            BackendCommand::StopPlayback => loadercmd_sender.send(LoaderCmd::Stop).unwrap(),
            BackendCommand::Shutdown => {
                // returning will drop loadercmd_sender, which will close
                // the channel, and cascade through the other running threads as well
                return;
            }
        }
    }
}

