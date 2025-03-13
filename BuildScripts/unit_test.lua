local engine_root_dir = engine_root_dir

task("unit_test")
do
    on_run(function()
        local folders = {
            "rs_artifact",
            "rs_assimp",
            "rs_audio",
            "rs_core_audio",
            "rs_core_minimal",
            "rs_editor",
            "rs_engine",
            "rs_foundation",
            "rs_hotreload_plugin",
            "rs_metis",
            "rs_proc_macros_test",
            "rs_quickjs",
            "rs_shader_compiler_core",
        }
        for k, v in ipairs(folders) do
            os.cd(path.join(engine_root_dir, v))
            os.exec("cargo test")
        end
    end)
    set_menu {
        usage = "xmake unit_test",
        description = "Unit test",
    }
end
