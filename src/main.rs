use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::{thread, time::SystemTime};

fn get_seconds_since_epoch() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HttpCheck {
    /// The domain this check will be hitting
    domain: String,
    /// Path on the domain to hit
    path: String,
    /// The port for the domain
    port: String,

    timeout_sec: u64,
    check_interval_sec: u64,
    healthy_threshold: u64,
    unhealthy_threshold: u64,

    #[serde(default = "default_fail_count")]
    fail_count: u64,
}

fn default_fail_count() -> u64 {
    0
}

impl HttpCheck {
    pub fn get_full_url(&self) -> String {
        format!("http://{}:{}{}", self.domain, self.port, self.path)
    }

    pub fn get_the_new_fail_count(&self) -> u64 {
        let new_fail_count = if self.is_ok() {
            if self.fail_count == 0 {
                0
            } else {
                self.fail_count - 1
            }
        } else {
            self.fail_count + 1
        };

        new_fail_count.clamp(0, self.unhealthy_threshold)
    }

    pub fn fail_count(&self) -> u64 {
        self.fail_count
    }

    pub fn set_the_new_fail_count(&mut self, new_fail_count: u64) {
        self.fail_count = new_fail_count;
    }

    fn is_ok(&self) -> bool {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_sec))
            .build()
            .unwrap();

        match client.get(self.get_full_url()).send() {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.fail_count < self.healthy_threshold
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_file = match args.get(1) {
        Some(file) => file,
        None => panic!("No config file provided"),
    };

    let config_content = match std::fs::read_to_string(config_file) {
        Ok(content) => content,
        Err(_) => panic!("Unable to read config file"),
    };

    let config_checks: Vec<HttpCheck> = match serde_json::from_str(&config_content) {
        Ok(checks) => checks,
        Err(e) => panic!("Unable to parse config file: {}", e),
    };

    let checks = Arc::new(Mutex::new(config_checks));

    let thread_mutex = Arc::clone(&checks);
    thread::spawn(move || {
        let mut last_run = get_seconds_since_epoch();
        loop {
            let now = get_seconds_since_epoch();

            if now - last_run < 1 {
                continue;
            }

            last_run = now;
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!(" {} Checking heath", now);
            println!("");
            for (i, check) in thread_mutex.lock().unwrap().iter().enumerate() {
                if check.is_healthy() {
                    println!("  {}) {} is healthy", i + 1, check.get_full_url());
                } else {
                    println!("  {}) {} is unhealthy", i + 1, check.get_full_url());
                }
            }
        }
    });

    let mutex = Arc::clone(&checks);
    let mut last_run = get_seconds_since_epoch();
    loop {
        let now = get_seconds_since_epoch();

        if now - last_run < 1 {
            continue;
        }

        last_run = now;

        let cloned_checks = mutex.lock().unwrap().clone();

        for (i, check) in cloned_checks.iter().enumerate() {
            if now % check.check_interval_sec == 0 {
                let new_fail_count = check.get_the_new_fail_count();
                if check.fail_count() != new_fail_count {
                    mutex.lock().unwrap()[i].set_the_new_fail_count(new_fail_count);
                }
            }
        }
    }
}
