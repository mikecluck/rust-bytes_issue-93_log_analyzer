// Rust Bytes Challenge Issue #93 Log Analyzer
mod logs;
mod parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run <path-to-file>");
        std::process::exit(1);
    }

    let file = std::fs::File::open(args[1].clone()).unwrap();
    let stats = parser::parse(file);
    std::fs::write(
        "summary.json",
        serde_json::ser::to_string_pretty(&stats).unwrap(),
    )
    .unwrap();
    println!("Results written to ./summary.json");
}

#[cfg(test)]
mod test {
    use crate::logs::{LogEntry, LogStatsBuilder};

    #[test]
    fn handle_one_log_entry() {
        let mut stats = LogStatsBuilder::new();
        stats.add_log_entry(LogEntry {
            timestamp: "Jul 1 09:00:55".into(),
            hostname: "calvisitor-10-105-160-95".into(),
            process_name: "kernel".into(),
            pid: 43,
            message: "Alfa Bravo Charlie Delta Echo".into(),
        });
        let stats = stats.build();

        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.by_process["kernel"], 1);
        assert_eq!(stats.by_hostname["calvisitor-10-105-160-95"], 1);
        assert_eq!(stats.most_frequent_process, "kernel");
        assert_eq!(stats.most_frequent_hostname, "calvisitor-10-105-160-95");
        assert_eq!(
            stats.top_keywords,
            vec!["alfa", "bravo", "charlie", "delta", "echo"]
        );

        let mut stats = LogStatsBuilder::new();
        stats.add_log_entry(LogEntry {
            timestamp: "Jul 1 09:00:55".into(),
            hostname: "home".into(),
            process_name: "netbiosd".into(),
            pid: 43,
            message: "Alfa Bravo Charlie Alfa Bravo".into(),
        });
        let stats = stats.build();

        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.by_process["netbiosd"], 1);
        assert_eq!(stats.by_hostname["home"], 1);
        assert_eq!(stats.most_frequent_process, "netbiosd");
        assert_eq!(stats.most_frequent_hostname, "home");
        assert_eq!(stats.top_keywords, vec!["alfa", "bravo"]);
    }

    #[test]
    fn handle_two_entries() {
        let mut stats = LogStatsBuilder::new();
        stats.add_log_entry(LogEntry {
            timestamp: "Jul 1 09:00:55".into(),
            hostname: "hostname_1".into(),
            process_name: "process_1".into(),
            pid: 1,
            message: "Alfa Charlie".into(),
        });
        stats.add_log_entry(LogEntry {
            timestamp: "Jul 1 09:00:55".into(),
            hostname: "hostname_1".into(),
            process_name: "process_1".into(),
            pid: 1,
            message: "Alfa Bravo".into(),
        });
        let stats = stats.build();

        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.by_process["process_1"], 2);
        assert_eq!(stats.by_hostname["hostname_1"], 2);
        assert_eq!(stats.most_frequent_process, "process_1");
        assert_eq!(stats.most_frequent_hostname, "hostname_1");
        assert_eq!(stats.top_keywords, vec!["alfa"]);

        let mut stats = LogStatsBuilder::new();
        stats.add_log_entry(LogEntry {
            timestamp: "Jul 1 09:00:55".into(),
            hostname: "hostname_1".into(),
            process_name: "process_1".into(),
            pid: 1,
            message: "Alfa Charlie".into(),
        });
        stats.add_log_entry(LogEntry {
            timestamp: "Jul 1 09:00:55".into(),
            hostname: "hostname_1".into(),
            process_name: "process_2".into(),
            pid: 1,
            message: "Bravo Delta".into(),
        });
        let stats = stats.build();

        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.by_process["process_1"], 1);
        assert_eq!(stats.by_process["process_2"], 1);
        assert_eq!(stats.by_hostname["hostname_1"], 2);
        assert_eq!(stats.most_frequent_process, "process_1");
        assert_eq!(stats.most_frequent_hostname, "hostname_1");
        assert_eq!(
            stats.top_keywords,
            vec!["alfa", "bravo", "charlie", "delta"]
        );
    }
}
