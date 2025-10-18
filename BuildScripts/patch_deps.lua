local engine_root_dir = engine_root_dir

task("patch_deps")
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        config.load()
        os.cd(path.join(engine_root_dir, "rs_editor"))
        os.exec("cargo update timezone_provider --precise 0.0.14")
        os.cd(path.join(engine_root_dir, "rs_desktop_standalone"))
        os.exec("cargo update timezone_provider --precise 0.0.14")
    end)
    set_menu {
        usage = "xmake patch_deps",
        description = "Patch dependencies",
        options = {
        }
    }
