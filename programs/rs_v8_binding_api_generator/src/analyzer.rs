use ra_ap_ide::RootDatabase;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace};
use ra_ap_proc_macro_api::ProcMacroClient;
use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource};
use ra_ap_vfs::{AbsPathBuf, Vfs};
use std::num::NonZero;

pub struct Analyzer {
    pub root_database: RootDatabase,
    pub vfs: Vfs,
    pub proc_macro_server: Option<ProcMacroClient>,
}

impl Analyzer {
    pub fn new(manifest_file_path: &std::path::Path) -> anyhow::Result<Analyzer> {
        let mut cargo_config = CargoConfig::default();
        cargo_config.sysroot = Some(RustLibSource::Discover);
        let project_manifest = ProjectManifest::from_manifest_file(AbsPathBuf::assert_utf8(
            manifest_file_path.to_path_buf(),
        ))?;
        let project_workspace = ProjectWorkspace::load(
            project_manifest.clone(),
            &cargo_config,
            &|message: String| {
                let _ = message;
            },
        )?;
        let num_worker_threads = std::thread::available_parallelism()
            .unwrap_or(NonZero::new(1).unwrap())
            .get();
        let proc_macro_processes = std::thread::available_parallelism()
            .unwrap_or(NonZero::new(1).unwrap())
            .get();
        let load_cargo_config: LoadCargoConfig = LoadCargoConfig {
            load_out_dirs_from_check: true,
            with_proc_macro_server: ProcMacroServerChoice::None,
            prefill_caches: false,
            num_worker_threads,
            proc_macro_processes,
        };
        let (root_database, vfs, proc_macro_server) = load_workspace(
            project_workspace.clone(),
            &cargo_config.extra_env,
            &load_cargo_config,
        )?;

        Ok(Analyzer {
            root_database,
            vfs,
            proc_macro_server,
        })
    }
}
