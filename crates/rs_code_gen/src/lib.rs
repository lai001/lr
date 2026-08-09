use ra_ap_ide::RootDatabase;

pub fn resolve_trait_import_path(db: &RootDatabase, rs_trait: &ra_ap_hir::Trait) -> String {
    let name = rs_trait.name(db);
    let module = rs_trait.module(db);
    let krate = module.krate(db);
    let mut modules = vec![];
    let mut current_module = Some(module.clone());
    while let Some(module) = current_module {
        modules.push(module);
        current_module = module.parent(db);
    }
    let crate_name = krate
        .display_name(db)
        .map(|x| x.crate_name().symbol().as_str().to_string());
    let mut module_chain = modules
        .iter()
        .flat_map(|x| x.name(db).map(|x| x.as_str().to_string()))
        .collect::<Vec<String>>();
    module_chain.reverse();
    if let Some(crate_name) = crate_name {
        module_chain.insert(0, crate_name);
    }
    module_chain.push(name.as_str().to_string());
    module_chain.join("::")
}

pub fn implemented_traits(
    db: &RootDatabase,
    rs_struct: &ra_ap_hir::Struct,
) -> Vec<ra_ap_hir::Trait> {
    ra_ap_hir::Impl::all_for_type(db, rs_struct.ty(db))
        .into_iter()
        .filter_map(|impl_| impl_.trait_(db))
        .collect::<Vec<_>>()
}

pub fn implemented_trait_paths(
    db: &ra_ap_ide::RootDatabase,
    implemented_traits: Vec<ra_ap_hir::Trait>,
) -> Vec<String> {
    let implemented_trait_paths = implemented_traits
        .iter()
        .map(|t| resolve_trait_import_path(db, t))
        .collect::<Vec<String>>();
    implemented_trait_paths
}

pub fn is_generic_function(db: &ra_ap_ide::RootDatabase, function: ra_ap_hir::Function) -> bool {
    let generic_def: ra_ap_hir::GenericDef = function.into();
    let has_type_generics = generic_def
        .params(db)
        .iter()
        .any(|p| matches!(p, ra_ap_hir::GenericParam::TypeParam(_)));
    if has_type_generics {
        return true;
    }
    false
}
