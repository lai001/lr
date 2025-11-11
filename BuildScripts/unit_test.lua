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
            "rs_v8_host",
            "crates/rs_render_core",
            "crates/rs_pack",
            "crates/rs_text",
        }
        for k, v in ipairs(folders) do
            os.cd(path.join(engine_root_dir, v))
            local package = path.basename(v)
            os.exec(format("cargo test --package %s", package))
            os.exec(format("cargo test --package %s --release", package))
        end

        local with_additional_args = {
            ["rs_metis"] = { features = { "required_detail_edges" } }
        }
        for k, v in pairs(with_additional_args) do
            os.cd(path.join(engine_root_dir, k))
            local features = v["features"]
            local features_args = ""
            for k, v in ipairs(features) do
                features_args = format("%s --features %s", features_args, v)
            end
            local package = path.basename(k)
            os.exec(format("cargo test --package %s %s", package, features_args))
            os.exec(format("cargo test --package %s %s --release", package, features_args))
        end
    end)
    set_menu {
        usage = "xmake unit_test",
        description = "Unit test",
    }
end
