local engine_root_dir = engine_root_dir
local ffmpeg_dir = ffmpeg_dir
local russimp_prebuild_dir = russimp_prebuild_dir

task("ci")
do
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        import("core.project.task")
        config.load()
        local extra_envs = {
            ["FFMPEG_DIR"] = ffmpeg_dir,
            ["RUSSIMP_PACKAGE_DIR"] = russimp_prebuild_dir
        }
        local tracy_android_libs_envs = {
            ["TRACY_CLIENT_LIB_PATH"] = path.join(engine_root_dir, "/build/android/arm64-v8a/debug"),
            ["TRACY_CLIENT_LIB"] = "tracy-client",
            ["TRACY_CLIENT_STATIC"] = 1
        }
        local function build_feature_args(features)
            local args = ""
            for k, v in ipairs(features) do
                args = args .. " --features " .. v
            end
            return args
        end
        local editor_feature_args = build_feature_args({
            "editor", "renderdoc", "plugin_shared_crate", "plugin_dotnet", "plugin_v8", "network"
        })
        local standalone_feature_args = build_feature_args({
            "standalone", "plugin_shared_crate", "plugin_v8"
        })
        os.addenvs(extra_envs)
        os.cd(path.join(engine_root_dir, "rs_editor"))
        os.exec("cargo build --package rs_editor --bin editor" .. editor_feature_args)
        os.exec("cargo build --package rs_editor --bin editor --release" .. editor_feature_args)
        os.cd(path.join(engine_root_dir, "build/target/release"))
        os.exec("./rs_shader_compiler.exe")
        os.cd(path.join(engine_root_dir, "rs_desktop_standalone"))
        os.exec("cargo build --package rs_desktop_standalone --bin rs_desktop_standalone" .. standalone_feature_args)
        os.exec("cargo build --package rs_desktop_standalone --bin rs_desktop_standalone --release" .. standalone_feature_args)
        os.cd(engine_root_dir)
        os.addenvs(tracy_android_libs_envs)
        os.exec("xmake build_android_target --mode=debug --target=aarch64-linux-android")
        local envs = os.getenvs()
        envs["TRACY_CLIENT_LIB_PATH"] = path.join(engine_root_dir, "/build/android/arm64-v8a/release")
        os.setenvs(envs)
        os.exec("xmake build_android_target --mode=release --target=aarch64-linux-android")
    end)
    set_menu {
        usage = "xmake ci",
    }
end
