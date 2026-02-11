use std::path::Path;

use crate::config::Config;
use crate::error::Result;

pub fn run(host: &str, port: u16) -> Result<()> {
    let config = Config::load(Path::new("."))?;
    let output_dir = config.build.output_dir.clone();

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| crate::error::Error::Render(format!("failed to start async runtime: {e}")))?;

    rt.block_on(async { crate::server::start(host, port, &output_dir).await })
}
