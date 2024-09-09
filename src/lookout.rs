use std::path::Path;
use sysinfo::{Pid, Process, ProcessesToUpdate, System};
use tokio::time::{sleep, Duration};

pub struct Lookout {
    system: System,
    osu_pid: Option<Pid>,
}

fn get_osu_lazer_pid(system: &mut System) -> Option<Pid> {
    system.refresh_processes(ProcessesToUpdate::All);
    system
        .processes_by_name("osu!".as_ref())
        .find(|&process| {
            process
                .exe()
                .and_then(|path| path.to_str())
                .map(|s| s.contains("lazer"))
                .unwrap_or(false)
        })
        .map(|process| process.pid())
}

impl Lookout {
    fn new() -> Self {
        let system = System::new_all();
        Self {
            system,
            osu_pid: None,
        }
    }

    fn run_time(&self) -> Option<u64> {
        self.osu_pid
            .map(|pid| self.system.process(pid).unwrap().run_time())
    }

    pub async fn watch_start(&mut self) -> Pid {
        loop {
            if let Some(pid) = get_osu_lazer_pid(&mut self.system) {
                self.osu_pid = Some(pid);
                return pid;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn watch_stop(&mut self) {
        loop {
            if let Some(pid) = self.osu_pid {
                self.system
                    .refresh_processes(ProcessesToUpdate::Some(&[pid]));
                if self.system.process(pid).is_none() {
                    self.osu_pid = None;
                    return;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
}
