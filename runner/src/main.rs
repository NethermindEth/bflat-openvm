use std::path::PathBuf;
use openvm_sdk::{config::{AggregationSystemParams, AppConfig}, GenericSdk, StdIn};
use openvm_sdk_config::SdkVmConfig;
use openvm_stark_sdk::config::{
    app_params_with_100_bits_security, baby_bear_poseidon2::BabyBearPoseidon2CpuEngine,
    MAX_APP_LOG_STACKED_HEIGHT,
};

mod hints;
use hints::{RunnerVmConfig, RunnerVmCpuBuilder};

/// The SDK on the standard configuration plus the division hint (see hints.rs).
type Sdk = GenericSdk<BabyBearPoseidon2CpuEngine, RunnerVmCpuBuilder>;

fn main() -> eyre::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 3 { eyre::bail!("usage: openvm-runelf <elf> <input-bytes>"); }
    // SdkVmConfig::standard(), the configuration that the accelerators of
    // accel-rs init.rs are in the order of, and that ere uses. The guest needs
    // no openvm.toml.
    let cfg = AppConfig::new(
        RunnerVmConfig(SdkVmConfig::standard()),
        app_params_with_100_bits_security(MAX_APP_LOG_STACKED_HEIGHT),
    );
    let sdk = Sdk::new(cfg, AggregationSystemParams::default())?;
    let mut stdin = StdIn::default();
    stdin.write_bytes(&std::fs::read(&a[2])?);
    let t = std::time::Instant::now();
    let out = sdk.compile_and_execute(PathBuf::from(&a[1]), stdin)?;
    eprintln!("executed in {:?}", t.elapsed());
    println!("public_values={}", hex::encode(out));
    Ok(())
}
