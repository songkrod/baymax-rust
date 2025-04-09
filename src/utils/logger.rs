use flexi_logger::{Logger, Cleanup, Criterion, Naming, Duplicate, FileSpec, Record, DeferredNow};
use crate::utils::config::Config;
use std::io::Write;
use std::convert::TryInto;

/// เริ่มระบบ logger พร้อม config และรูปแบบ timestamp + level
pub fn init_logger(config: &Config) {
    Logger::try_with_str(&config.log_level)
        .unwrap_or_else(|_| {
            panic!(
                "Invalid log level '{}' in config. Use 'info', 'debug', etc.",
                config.log_level
            )
        })
        .log_to_file(
            FileSpec::default()
                .directory(&config.log_dir)
                .basename(&config.log_file_name)
                .suffix("log"),
        )
        .format_for_stdout(custom_log_format)
        .format_for_files(custom_log_format)
        .rotate(
            Criterion::Size(
                config
                    .log_rotate_size
                    .try_into()
                    .expect("log_rotate_size must fit into u64"),
            ),
            Naming::Numbers,
            Cleanup::KeepLogFiles(config.log_rotate_count),
        )
        .duplicate_to_stdout(Duplicate::All) // ✅ แสดง stdout ด้วย
        .start()
        .expect("Failed to initialize logger");

    log::info!("✅ Logger initialized with config: {:?}", config);
}

/// รูปแบบ log: [2025-04-07 22:11:53] [INFO] ข้อความ...
fn custom_log_format(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &Record,
) -> std::io::Result<()> {
    write!(
        w,
        "[{}] [{:5}] {}",
        now.now().format("%Y-%m-%d %H:%M:%S"),
        record.level(),
        &record.args()
    )
}