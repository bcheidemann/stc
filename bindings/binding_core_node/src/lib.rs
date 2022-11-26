extern crate swc_node_base;

use std::{path::PathBuf, sync::Arc};

use napi_derive::napi;
use stc_ts_builtin_types::Lib;
use stc_ts_env::{Env, ModuleConfig, Rule};
use stc_ts_file_analyzer::env::EnvFactory;
use stc_ts_module_loader::resolvers::node::NodeResolver;
use stc_ts_type_checker::Checker;
use swc_common::{
    errors::{ColorConfig, EmitterWriter, Handler},
    FileName, SourceMap,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::TsConfig;

#[napi]
pub fn check(path: String) -> Vec<stc_ts_errors::JsError> {
    let cm = Arc::new(SourceMap::default());
    let handler = {
        let emitter = Box::new(EmitterWriter::stderr(ColorConfig::Always, Some(cm.clone()), false, false));
        Arc::new(Handler::with_emitter(true, false, emitter))
    };

    let libs = {
        let mut libs = Lib::load("es5");

        // Currently this is not needed since only one libs is loaded
        libs.sort();
        libs.dedup();

        libs
    };

    let env = Env::simple(Rule { ..Default::default() }, EsVersion::latest(), ModuleConfig::None, &libs);

    let path = PathBuf::from(path);

    {
        let checker = Checker::new(
            cm.clone(),
            handler.clone(),
            env.clone(),
            TsConfig { ..Default::default() },
            None,
            Arc::new(NodeResolver),
        );

        checker.load_typings(&path, None, None);
    }

    let mut errors = vec![];

    {
        let mut checker = Checker::new(
            cm.clone(),
            handler.clone(),
            env,
            TsConfig { ..Default::default() },
            None,
            Arc::new(NodeResolver),
        );

        checker.check(Arc::new(FileName::Real(path)));

        errors.extend(checker.take_errors());
    }

    errors.iter().map(|err| err.into()).collect()
}
