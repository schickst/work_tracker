use std::process::Command;
use std::thread;
use chrono::NaiveDateTime;
use clap::{crate_version, crate_authors, crate_description, Arg, App};


fn parse_and_trim(stdout: Vec<u8>) -> String {
    let mut data = String::from_utf8(stdout).expect("Unable to parse stdout");

    let len = data.trim_end_matches(&['\r', '\n'][..]).len();
    data.truncate(len);
    data
}


fn get_active_window_pid() -> Option<u64> {
    let output = Command::new("xdotool")
        .arg("getactivewindow")
        .arg("getwindowpid")
        .output()
        .expect("Failed to get active window pid");
    //println!("{:?}", output);
    if output.status.success() {
        let data = parse_and_trim(output.stdout);
        let pid = data.parse::<u64>().unwrap();
        Some(pid)
    } else {
        let error = parse_and_trim(output.stdout);
        println!("{}", error);
        None
    }    
}

fn get_active_window_name() -> String {
    let output = Command::new("xdotool")
        .arg("getactivewindow")
        .arg("getwindowname")
        .output()
        .expect("Failed to get active window name");
    //println!("{:?}", output); 
    parse_and_trim(output.stdout)
}


fn get_active_application_title(pid: u64) -> String {
    //ps -p 3783 -o comm=
    let output = Command::new("ps")
        .arg("-p")
        .arg(pid.to_string())
        .arg("-o")
        .arg("comm=")
        .output()
        .expect("Failed to get active window name");
    //println!("{:?}", output);
    parse_and_trim(output.stdout)
}



fn print_summary(applications: &[String], windows: &[LogEntry]) {
    println!();
    println!("Summary");
    println!("=======");
    println!();

    for application in applications {

        let application_times = windows.iter()
            .filter(|o| o.application_title == *application)
            .map(|o| o.duration_secs)
            .collect::<Vec<i64>>();
        let application_time: i64 = application_times.iter().sum();
        let application_duration = NaiveDateTime::from_timestamp(application_time, 0);

        println!("------------------------------------------------------------");
        println!("{} | {}", application_duration.format("%H:%M:%S"), application);
        println!("------------------------------------------------------------");

        for window in windows {
            if window.application_title != *application {
                continue;
            }

            let duration = NaiveDateTime::from_timestamp(window.duration_secs, 0);
            println!("\t{} | {}", duration.format("%H:%M:%S"), window.window_name);
        }
    }
    println!();
}


fn track_window(pid: u64, applications: &mut Vec<String>, windows: &mut Vec<LogEntry>, period: i64) {
    let name = get_active_window_name();
    let title = get_active_application_title(pid);

    if !applications.contains(&title) {
        applications.push(title.clone());
    }

    match windows.iter_mut().find(|o| o.window_name == name) {
        Some(window) => {
            window.duration_secs += period;
        },
        None => {
            let new_entry = LogEntry { 
                application_title: title,
                window_name: name.clone(),
                duration_secs: period
            };
            windows.push(new_entry);
        }
    }
}



#[derive(Debug, PartialEq, Clone)]
struct LogEntry {
    application_title: String,
    window_name: String,
    duration_secs: i64
}



fn main() {
    let matches = App::new("Work Tracker")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(Arg::with_name("resolution")
            .short("r")
            .long("resolution")
            .value_name("SECONDS")
            .help("Sets a custom tracking resolution in seconds")
            .takes_value(true))
        .get_matches();

    let resolution_parameter = matches.value_of("resolution").unwrap_or("5");
    let resolution = resolution_parameter.parse::<i64>().unwrap();

    let mut applications: Vec<String> = Vec::new();
    let mut windows: Vec<LogEntry> = Vec::new();

    loop {
        let active_window = get_active_window_pid();

        if let Some(pid) = active_window {
            track_window(pid, &mut applications, &mut windows, resolution);
        }

        print_summary(&applications, &windows);

        let delay = std::time::Duration::from_secs(resolution as u64);
        thread::sleep(delay);
    }

    
}
