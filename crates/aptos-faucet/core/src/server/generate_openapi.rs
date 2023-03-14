// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    checkers::CaptchaManager,
    endpoints::{build_openapi_service, BasicApi, CaptchaApi, FundApi, FundApiComponents},
    funder::FakeFunder,
};
use anyhow::Result;
use clap::{ArgEnum, Parser};
use futures::lock::Mutex;
use std::{path::PathBuf, sync::Arc};

#[derive(Clone, Debug, Parser)]
pub struct GenerateOpenapi {
    #[clap(flatten)]
    output_args: OutputConfig,
}

impl GenerateOpenapi {
    pub async fn generate_openapi(&self) -> Result<()> {
        let funder = Arc::new(FakeFunder);
        let fund_api = FundApi {
            components: Arc::new(FundApiComponents {
                bypassers: Vec::new(),
                checkers: Vec::new(),
                funder: funder.clone(),
                return_rejections_early: true,
                concurrent_requests_semaphore: None,
            }),
        };

        let api_service = build_openapi_service(
            BasicApi {
                concurrent_requests_semaphore: None,
                funder,
            },
            CaptchaApi {
                enabled: false,
                captcha_manager: Arc::new(Mutex::new(CaptchaManager::new())),
            },
            fund_api,
        );

        let spec = match self.output_args.format {
            OutputFormat::Json => api_service.spec(),
            OutputFormat::Yaml => api_service.spec_yaml(),
        };
        self.output_args.write(&spec)
    }
}

#[derive(ArgEnum, Clone, Debug)]
pub enum OutputFormat {
    Json,
    Yaml,
}

#[derive(Clone, Debug, Parser)]
pub struct OutputConfig {
    /// By default, the spec is written to stdout. If this is provided, the
    /// tool will instead write the spec to the provided path.
    #[clap(short, long)]
    pub output_path: Option<PathBuf>,

    /// What format to output the spec in.
    #[clap(short, long, arg_enum, default_value = "yaml")]
    pub format: OutputFormat,
}

impl OutputConfig {
    pub fn write(&self, output: &str) -> Result<()> {
        match &self.output_path {
            Some(path) => std::fs::write(path, output)?,
            None => println!("{}", output),
        }
        Ok(())
    }
}