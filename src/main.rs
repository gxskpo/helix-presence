use core::time;
use discord_presence::Client;
use dotenv::dotenv;
use std::thread;
use std::time::SystemTime;
use sysinfo::Pid;
use sysinfo::System;
use time::Duration;

fn humanize_duration(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    if seconds < 60 {
        return format!("00:{seconds:02} elapsed");
    }

    let days = seconds / (60 * 60 * 24);
    let hours = (seconds / (60 * 60)) % 24;
    let minutes = (seconds / 60) % 60;
    let seconds = seconds % 60;

    if days > 0 {
        format!("{days:02}:{hours:02}:{minutes:02}:{seconds:02} elapsed",)
    } else if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02} elapsed")
    } else {
        format!("{minutes:02}:{seconds:02} elapsed")
    }
}

fn update_presence(pid: u32, app_id: u64) {
    let mut system = System::new_all();
    let mut started_at: Option<SystemTime> = None;

    let mut drpc = Client::new(app_id);
    drpc.start();

    thread::sleep(Duration::from_secs(2));

    let pd = Pid::from_u32(pid);
    loop {
        system.refresh_processes();
        if system.processes().get(&pd).is_none() {
            break;
        }
        if started_at.is_none() {
            started_at = Some(SystemTime::now());
        }

        let elapsed = humanize_duration(started_at.unwrap().elapsed().unwrap());
        drpc.set_activity(|act| {
            act.assets(|assets| assets.large_image("logo").large_text("Helix Editor"))
                .state(elapsed)
        })
        .expect("Failed to set activity");
        thread::sleep(Duration::from_millis(200));
    }

    drpc.clear_activity().expect("Failed to clear activity");
    drpc.shutdown().expect("Failed to shutdown");
}

fn get_pid() -> Option<u32> {
    let sys = System::new_all();
    let mut pd: Option<&Pid> = None;
    for (pid, process) in sys.processes() {
        if process.name().eq("hx") {
            pd = Some(pid);
        }
    }

    if let Some(pd) = pd {
        return Some(pd.as_u32());
    }
    None
}

fn main() {
    dotenv().ok();
    let app_id: u64 = 1_119_080_971_879_321_621;

    loop {
        let Some(pid) = get_pid() else {
            thread::sleep(Duration::from_secs(3));
            continue;
        };

        update_presence(pid, app_id);
    }
}
