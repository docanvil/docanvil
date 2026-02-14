use std::path::Path;

use crate::config::Config;
use crate::error::Result;

pub fn run(host: &str, port: u16, project_root: &Path) -> Result<()> {
    let config = Config::load(project_root)?;
    let output_dir = project_root.join(&config.build.output_dir);

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| crate::error::Error::Render(format!("failed to start async runtime: {e}")))?;

    rt.block_on(async { crate::server::start(host, port, &output_dir, project_root).await })
}
