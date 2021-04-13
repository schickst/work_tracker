use std::collections::HashMap;
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

    parse_and_trim(output.stdout)
}



fn print_summary(app_windows: &HashMap<String, HashMap<String, i64>>) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen reset cursor
    println!("Summary");
    println!("=======");
    println!();

    for (application, windows) in app_windows {
        let application_time: i64 = windows.values().sum();
        let application_duration = NaiveDateTime::from_timestamp(application_time, 0);

        println!("------------------------------------------------------------");
        println!("{} | {}", application_duration.format("%H:%M:%S"), application);
        println!("------------------------------------------------------------");
        
        let mut sorted_windows: Vec<(&String, &i64)> = windows.iter().collect();
        sorted_windows.sort_by(|a, b| b.1.cmp(a.1));

        for (window, duration_secs) in sorted_windows {
            let duration = NaiveDateTime::from_timestamp(*duration_secs, 0);
            println!("\t{} | {}", duration.format("%H:%M:%S"), window);
        }
    }
}


fn track_window(pid: u64, app_windows: &mut HashMap<String, HashMap<String, i64>>, period: i64) {
    let name = get_active_window_name();
    let title = get_active_application_title(pid);

    if !app_windows.contains_key(&title) {
        app_windows.insert(title.clone(), HashMap::new());
    }

    // We ensured it exists above
    let windows = app_windows.get_mut(&title).unwrap();

    if let Some(window) = windows.get_mut(&name) {
        *window += period;
    } else {
        windows.insert(name, period);
    }
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

    // HashMap<Application, HashMap<Window, Duration>>
    let mut app_windows : HashMap<String, HashMap<String, i64>> = HashMap::new();

    loop {
        let active_window = get_active_window_pid();

        if let Some(pid) = active_window {
            track_window(pid, &mut app_windows, resolution);
        }

        print_summary(&app_windows);

        let delay = std::time::Duration::from_secs(resolution as u64);
        thread::sleep(delay);
    }    
}
