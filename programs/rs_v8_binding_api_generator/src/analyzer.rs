use ra_ap_hir::db::DefDatabase;
use ra_ap_ide::RootDatabase;
use ra_ap_ide_db::base_db::SourceDatabase;
use ra_ap_load_cargo::{load_workspace, LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_proc_macro_api::ProcMacroServer;
use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustLibSource};
use ra_ap_vfs::{AbsPathBuf, Vfs};

pub struct Analyzer {
    pub root_database: RootDatabase,
    pub vfs: Vfs,
    pub proc_macro_server: Option<ProcMacroServer>,
}

impl Analyzer {
    pub fn new(crate_name: &str) -> anyhow::Result<Analyzer> {
        let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();
        let workspace_dir = engine_root_dir;
        let manifest_file_path = workspace_dir.join(format!("{crate_name}/Cargo.toml"));
        let mut cargo_config = CargoConfig::default();
        cargo_config.sysroot = Some(RustLibSource::Discover);
        let project_manifest = ProjectManifest::from_manifest_file(AbsPathBuf::assert_utf8(
            manifest_file_path.clone(),
        ))?;
        let project_workspace = ProjectWorkspace::load(
            project_manifest.clone(),
            &cargo_config,
            &|message: String| {
                let _ = message;
            },
        )?;
        let load_cargo_config: LoadCargoConfig = LoadCargoConfig {
            load_out_dirs_from_check: true,
            with_proc_macro_server: ProcMacroServerChoice::None,
            prefill_caches: false,
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

    pub fn run(&mut self) -> anyhow::Result<()> {
        let db = &mut self.root_database;
        let vfs = &mut self.vfs;
        // let crate_workspace_data = db.crate_workspace_data();
        let crate_graph = db.crate_graph();
        let crates = crate_graph.crates_in_topological_order();

        for krate in crates {
            let krate_data = crate_graph[krate].clone();
            let display_name = krate_data
                .display_name
                .map(|x| x.to_string())
                .unwrap_or_default();
            if display_name.starts_with("rs_") {
                // let crate_workspace_data = crate_workspace_data[&krate].clone();
                let def_map = db.crate_def_map(krate);
                for (_, module_data) in def_map.modules.iter() {
                    // let declaration_source = module_data.declaration_source(db);
                    // if let Some(declaration_source) = declaration_source {
                    // let module_def = sema.to_module_def(&declaration_source.value);
                    // if let Some(module_def) = module_def {}
                    // }
                    let definition_source_file_id = module_data.definition_source_file_id();
                    let file_id = definition_source_file_id.file_id();
                    if let Some(editioned_file_id) = file_id {
                        let file_path = vfs.file_path(editioned_file_id.into());
                        log::trace!("{}, Crate {}", file_path, display_name);
                    }
                }
            }
        }
        Ok(())
    }
}
