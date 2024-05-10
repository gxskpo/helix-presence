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
        return format!("00:{:02} elapsed", seconds);
    }

    let days = seconds / (60 * 60 * 24);
    let hours = (seconds / (60 * 60)) % 24;
    let minutes = (seconds / 60) % 60;
    let seconds = seconds % 60;

    if days > 0 {
        format!(
            "{:02}:{:02}:{:02}:{:02} elapsed",
            days, hours, minutes, seconds
        )
    } else if hours > 0 {
        format!("{:01}:{:02}:{:02} elapsed", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02} elapsed", minutes, seconds)
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
            act.assets(|ass| ass.large_image("logo").large_text("Helix Editor"))
                .state(elapsed)
        })
        .expect("Failed to set activity");
        thread::sleep(Duration::from_millis(200))
    }

    match drpc.clear_activity() {
        Err(_) => panic!("Failed to clear activity"),
        _ => (),
    }

    match drpc.shutdown() {
        Err(_) => panic!("Failed to close connection"),
        _ => (),
    }
}

fn get_pid() -> Option<u32> {
    let sys = System::new_all();
    let mut pd: Option<&Pid> = None;
    for (pid, process) in sys.processes() {
        if process.name().eq("hx") {
            pd = Some(pid);
        }
    }

    if pd.is_some() {
        return Some(pd.unwrap().as_u32());
    }

    None
}

fn main() {
    dotenv().ok();
    let app_id: u64 = 1119080971879321621;

    loop {
        let Some(pid) = get_pid() else {
            thread::sleep(Duration::from_secs(3));
            continue;
        };

        update_presence(pid, app_id);
    }
}
