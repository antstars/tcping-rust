use chrono::{DateTime, Local};
use clap::Parser;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct PingConfig {
    host: String,
    port: u16,
    #[arg(short, long, default_value_t = 4)]
    count: u32,
    #[arg(short = 't', long, help = "Ping continuously until stopped (Ctrl+C)")]
    continuous: bool,
    #[arg(short = 'w', long, default_value_t = 2000)]
    timeout_ms: u64,
}

#[derive(Default)]
struct SessionStatistics {
    transmitted: u32,
    successful: u32,
    unsuccessful: u32,
    
    start_time: Option<DateTime<Local>>,
    end_time: Option<DateTime<Local>>,
    last_successful_time: Option<DateTime<Local>>,
    last_unsuccessful_time: Option<DateTime<Local>>,
    
    current_uptime_start: Option<DateTime<Local>>,
    longest_uptime_start: Option<DateTime<Local>>,
    longest_uptime_end: Option<DateTime<Local>>,
    longest_uptime_duration: Duration,
    
    total_uptime: Duration,
    
    rtt_min: Option<f64>,
    rtt_max: Option<f64>,
    rtt_sum: f64,
}

impl SessionStatistics {
    fn record_start(&mut self) {
        self.start_time = Some(Local::now());
    }

    fn record_end(&mut self) {
        self.end_time = Some(Local::now());
        self.finalize_uptime_period(Local::now());
    }

    fn record_success(&mut self, rtt_ms: f64, timestamp: DateTime<Local>) {
        self.transmitted += 1;
        self.successful += 1;
        self.last_successful_time = Some(timestamp);

        self.rtt_min = Some(self.rtt_min.map_or(rtt_ms, |min| min.min(rtt_ms)));
        self.rtt_max = Some(self.rtt_max.map_or(rtt_ms, |max| max.max(rtt_ms)));
        self.rtt_sum += rtt_ms;

        if self.current_uptime_start.is_none() {
            self.current_uptime_start = Some(timestamp);
        }
    }

    // 独立处理失败逻辑，避免与成功逻辑耦合，确保状态流转清晰
    fn record_failure(&mut self, timestamp: DateTime<Local>) {
        self.transmitted += 1;
        self.unsuccessful += 1;
        self.last_unsuccessful_time = Some(timestamp);
        self.finalize_uptime_period(timestamp);
    }

    fn finalize_uptime_period(&mut self, end_time: DateTime<Local>) {
        if let Some(start) = self.current_uptime_start {
            let duration = (end_time - start).to_std().unwrap_or(Duration::ZERO);
            self.total_uptime += duration;

            if duration > self.longest_uptime_duration {
                self.longest_uptime_duration = duration;
                self.longest_uptime_start = Some(start);
                self.longest_uptime_end = Some(end_time);
            }
            self.current_uptime_start = None;
        }
    }
}

fn main() {
    let config = PingConfig::parse();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // 劫持 SIGINT，接管生命周期控制权以保证即使强制中断也能输出完整统计报告
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    execute_tcp_ping(&config, running);
}

fn execute_tcp_ping(config: &PingConfig, running: Arc<AtomicBool>) {
    let address = format!("{}:{}", config.host, config.port);
    let target_addr = match address.to_socket_addrs().and_then(|mut iter| {
        iter.next().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No IP found"))
    }) {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("Failed to resolve: {}", address);
            return;
        }
    };

    println!("TCPing {} on port {}", config.host, config.port);

    let mut stats = SessionStatistics::default();
    stats.record_start();
    
    let timeout = Duration::from_millis(config.timeout_ms);
    let interval = Duration::from_secs(1);
    let mut sequence = 1;

    while running.load(Ordering::SeqCst) && (config.continuous || sequence <= config.count) {
        let loop_start = Instant::now();
        let ping_timestamp = Local::now();

        match TcpStream::connect_timeout(&target_addr, timeout) {
            Ok(_) => {
                let latency_ms = loop_start.elapsed().as_secs_f64() * 1000.0;
                stats.record_success(latency_ms, ping_timestamp);
                println!(
                    "Reply from {} ({}) on port {} TCP_conn={} time={:.3} ms",
                    config.host, target_addr.ip(), config.port, sequence, latency_ms
                );
            }
            Err(_) => {
                stats.record_failure(ping_timestamp);
                println!("No response from {} ({}) on port {} TCP_conn={}", config.host, target_addr.ip(), config.port, sequence);
            }
        }

        sequence += 1;
        
        let elapsed = loop_start.elapsed();
        if elapsed < interval {
            let sleep_duration = interval - elapsed;
            let poll_interval = Duration::from_millis(50);
            let mut time_waited = Duration::ZERO;

            while time_waited < sleep_duration && running.load(Ordering::SeqCst) {
                let current_sleep = poll_interval.min(sleep_duration - time_waited);
                std::thread::sleep(current_sleep);
                time_waited += current_sleep;
            }
        }
    }

    stats.record_end();
    print_statistics(config, &stats);
}

fn print_statistics(config: &PingConfig, stats: &SessionStatistics) {
    let loss_rate = if stats.transmitted > 0 {
        (stats.unsuccessful as f64 / stats.transmitted as f64) * 100.0
    } else {
        0.0
    };

    let start = stats.start_time.unwrap();
    let end = stats.end_time.unwrap();
    let duration = end - start;
    
    // 采用与常规工具一致的停机时间估算：总持续时间减去连通时间
    let total_downtime = duration.to_std().unwrap_or(Duration::ZERO).saturating_sub(stats.total_uptime);

    let format_time = |t: Option<DateTime<Local>>| t.map_or("Never".to_string(), |t| t.format("%Y-%m-%d %H:%M:%S").to_string());
    let format_time_failed = |t: Option<DateTime<Local>>| t.map_or("Never failed".to_string(), |t| t.format("%Y-%m-%d %H:%M:%S").to_string());

    println!("\n--- {} TCPing statistics ---", config.host);
    println!("{} probes transmitted on port {} | {} received, {:.2}% packet loss", stats.transmitted, config.port, stats.successful, loss_rate);
    println!("successful probes:   {}", stats.successful);
    println!("unsuccessful probes: {}", stats.unsuccessful);
    println!("last successful probe:   {}", format_time(stats.last_successful_time));
    println!("last unsuccessful probe: {}", format_time_failed(stats.last_unsuccessful_time));
// 为什么：将毫秒转换为 f64 进行 round (四舍五入)，防止 2.9 秒被直接截断显示为 2 秒，以保证报表时间轴符合人类直观感受。
    let uptime_secs = (stats.total_uptime.as_millis() as f64 / 1000.0).round() as u64;
    let downtime_secs = (total_downtime.as_millis() as f64 / 1000.0).round() as u64;

    println!("total uptime:   {} seconds", uptime_secs);
    println!("total downtime: {} second{}", downtime_secs, if downtime_secs != 1 { "s" } else { "" });
    
    if let (Some(up_start), Some(up_end)) = (stats.longest_uptime_start, stats.longest_uptime_end) {
        let longest_uptime_secs = (stats.longest_uptime_duration.as_millis() as f64 / 1000.0).round() as u64;
        println!("longest consecutive uptime:   {} seconds from {} to {}", 
            longest_uptime_secs,
            up_start.format("%Y-%m-%d %H:%M:%S"),
            up_end.format("%Y-%m-%d %H:%M:%S")
        );
    } else {
        println!("longest consecutive uptime:   0 seconds");
    }

    if stats.successful > 0 {
        let avg = stats.rtt_sum / stats.successful as f64;
        println!("rtt min/avg/max: {:.3}/{:.3}/{:.3} ms", stats.rtt_min.unwrap_or(0.0), avg, stats.rtt_max.unwrap_or(0.0));
    }
    
    println!("--------------------------------------");
    println!("TCPing started at: {}", start.format("%Y-%m-%d %H:%M:%S"));
    println!("TCPing ended at:   {}", end.format("%Y-%m-%d %H:%M:%S"));
    
    let total_duration_secs = (duration.num_milliseconds() as f64 / 1000.0).round() as i64;
    let hours = total_duration_secs / 3600;
    let minutes = (total_duration_secs % 3600) / 60;
    let seconds = total_duration_secs % 60;
    println!("duration (HH:MM:SS): {:02}:{:02}:{:02}", hours, minutes, seconds);
}