use log::LevelFilter;
use rstest::*;
use crate::components::config::DEFAULT_CONFIG_YAML;
use super::*;

#[fixture]
fn init_logger() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Trace)
        .is_test(true)
        .try_init();
}

#[rstest]
#[case("mcr.microsoft.com/dotnet/runtime-deps:6.0-alpine-arm64v8",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/runtime-deps:6.0-alpine-arm64v8")]
#[case("mcr.microsoft.com/dotnet/runtime:6.0-alpine-arm64v8",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/runtime:6.0-alpine-arm64v8")]
#[case("mcr.microsoft.com/dotnet/sdk:6.0-alpine-arm64v8",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/sdk:6.0-alpine-arm64v8")]
#[case("mcr.microsoft.com/dotnet/aspnet:7.0-bullseye-slim",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/aspnet:7.0-bullseye-slim")]
#[case("mcr.microsoft.com/mssql/server:2017-CU12-ubuntu",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/server:2017-CU12-ubuntu")]
#[case("mcr.microsoft.com/java/jdk:11-zulu-ubuntu-19.10",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/jdk:11-zulu-ubuntu-19.10")]
#[case("mcr.microsoft.com/java/jre:11u10-zulu-ubuntu-18.04",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/jre:11u10-zulu-ubuntu-18.04")]
#[case("mcr.microsoft.com/windows:10.0.19042.1889-amd64",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/windows:10.0.19042.1889-amd64")]
#[case("mcr.microsoft.com/vscode/devcontainers/base:0-alpine-3.11",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/vscode_base:0-alpine-3.11")]
#[case("mcr.microsoft.com/vscode/devcontainers/rust:0",
"registry.cn-hangzhou.aliyuncs.com/newbe36524/vscode_rust:0")]
fn map_success(init_logger: (), #[case]source: &str, #[case]expected: &str) {
    let loader = ConfigLoader::default();
    let config = loader.load_config_yaml(DEFAULT_CONFIG_YAML).unwrap();
    let result = map_mirror_by_configuration(source, &config);
    assert!(result.is_ok(), "Failed to map image: {}, error: {}", source, result.err().unwrap());
    let result = result.unwrap();
    assert_eq!(result.mirror_image, expected);
}

pub const JSON1_YAML: &str = include_str!("json1.yaml");

#[rstest]
fn map_failed(init_logger: ()) {
    let loader = ConfigLoader::default();
    info!("{}",JSON1_YAML);
    let config = loader.load_config_yaml(JSON1_YAML).unwrap();
    let source = "mcr.microsoft.com/java/jdk:15u2-zulu-ubuntu-18.04";
    let result = map_mirror_by_configuration(source, &config);
    assert!(result.is_err());
}