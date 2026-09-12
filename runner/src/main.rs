use std::path::PathBuf;
use openvm_sdk::{config::{AggregationSystemParams, AppConfig}, Sdk, StdIn};
use openvm_sdk_config::SdkVmConfig;

fn main() -> eyre::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 4 { eyre::bail!("usage: openvm-runelf <elf> <openvm.toml> <input-bytes>"); }
    let cfg: AppConfig<SdkVmConfig> = toml::from_str(&std::fs::read_to_string(&a[2])?)?;
    let sdk = Sdk::new(cfg, AggregationSystemParams::default())?;
    let mut stdin = StdIn::default();
    stdin.write_bytes(&std::fs::read(&a[3])?);
    let t = std::time::Instant::now();
    let out = sdk.compile_and_execute(PathBuf::from(&a[1]), stdin)?;
    eprintln!("executed in {:?}", t.elapsed());
    println!("public_values={}", hex::encode(out));
    Ok(())
}
