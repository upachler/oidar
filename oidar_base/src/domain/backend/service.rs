use super::ports::*;
use std::{sync::mpsc::*, thread::JoinHandle, time::Duration};
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

pub fn new<L,D,P>() -> impl Backend 
where 
    L: Loader,
    D: Decoder,
    P: Player,
{
    let (command_sender, command_receiver) = sync_channel(10);
    let (event_sender, event_receiver) = sync_channel(10);
    let (loadercmd_sender, loadercmd_receiver) = sync_channel::<LoaderCmd>(10);

    let (chunk_sender, chunk_receiver) = sync_channel(2);
    std::thread::spawn(move ||{
        let mut loader = L::new();
        loop {
            
            let cmd;
            match loader.read_chunk() {
                Ok(Some(chunk)) => {
                    log::trace!("chunk received: {chunk:?}");
                    chunk_sender.send(chunk).unwrap();

                    cmd = match loadercmd_receiver.try_recv() {
                        Ok(cmd) => Some(cmd),
                        Err(TryRecvError::Disconnected) => return,
                        Err(TryRecvError::Empty) => None,
                    }
    
                }
                Ok(None) => {
                    log::trace!("loader has no active stream");

                    cmd = match loadercmd_receiver.recv() {
                        Ok(cmd) => Some(cmd),
                        Err(e)  => {
                            log::error!("error while receiving command: {e}");
                            return
                        },
                    }
    
                }
                Err(e) => {
                    log::error!("read_chunk() returned error '{e}', terminating loader thread");
                    break;
                },
            }

            if let Some(cmd) = cmd {
                match cmd {
                    LoaderCmd::SetUrl(url) => loader.set_url(Some(url)),
                    LoaderCmd::Stop => loader.set_url(None),
                }
            }
        }
    });

    let (frame_sender, frame_receiver) = sync_channel(2);
    std::thread::spawn(move ||{
        let mut decoder = D::new(chunk_receiver);
        loop {
            match decoder.decode() {
                Ok(frame) => frame_sender.send(frame).unwrap(),
                Err(_) => return,
            }
        }
    });

    std::thread::spawn(move ||{
        let mut player = P::new(frame_receiver);
        loop {
            const INTERVAL: Duration = Duration::from_millis(100);
            let start = std::time::Instant::now();
            if let Err(e) = player.play() {
                log::error!("error while playing frame: {e}")
            }
            // FIXME: We're currently using cpal for implementing the player,
            // which rolls it's own playback thread. Therefore, Player::play()
            // returns immediately (non-blocking), and calling it repeatedly
            // in  a loop sucks up lots of performance. So we call it in
            // 100ms intervals (which is not a long term solution)
            let naptime = INTERVAL - (std::time::Instant::now().duration_since(start));
            std::thread::sleep(naptime);
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

fn exec_core_thread(command_receiver: Receiver<BackendCommand>, loadercmd_sender: SyncSender<LoaderCmd>) 
{

    loop {
        log::debug!("waiting for command");
        let cmd = match command_receiver.recv() {
            Ok(cmd) => cmd,
            Err(_) => return,
        };

        log::debug!("command received: {cmd:?}");
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

