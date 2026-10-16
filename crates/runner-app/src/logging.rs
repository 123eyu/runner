pub use runner_core::logging::{startup_banner, startup_failure};

pub fn install(log_dir: &std::path::Path) -> anyhow::Result<()> {
    runner_core::logging::install(log_dir, "runner.log")
}
