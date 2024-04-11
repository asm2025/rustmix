use anyhow::Result;
use std::io::Write;

use rustmix::ai::Phi;

use super::*;

pub async fn test_phi() -> Result<()> {
    let phi = Phi::new().await?;

    let prompt = "The following is a 300 word essay about Paris:";
    print!("{}", prompt);
    std::io::stdout().flush()?;

    Ok(())
}
