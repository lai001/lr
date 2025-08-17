local engine_root_dir = engine_root_dir
local ffmpeg_dir = ffmpeg_dir
local russimp_prebuild_dir = russimp_prebuild_dir

task("ci")
    on_run(function()
        local function build_feature_args(features)
            local features_arg = ""
            for _, feature in ipairs(features) do
                features_arg = format("%s%s,", features_arg, feature)
            end
            return format("--features %s", features_arg)
        end
        local editor_feature_args = build_feature_args({
            "editor", "renderdoc", "plugin_shared_crate", "plugin_dotnet", "plugin_v8", "network"
        })
        local standalone_feature_args = build_feature_args({
            "standalone", "plugin_shared_crate", "plugin_v8"
        })
        os.cd(path.join(engine_root_dir, "build/target/release"))
        os.exec("./rs_shader_compiler.exe")
        os.cd(path.join(engine_root_dir, "rs_editor"))
        os.exec("cargo build --package rs_editor --bin editor " .. editor_feature_args)
        os.exec("cargo build --package rs_editor --bin editor --release " .. editor_feature_args)
        os.cd(path.join(engine_root_dir, "rs_desktop_standalone"))
        os.exec("cargo build --package rs_desktop_standalone --bin rs_desktop_standalone " .. standalone_feature_args)
        os.exec("cargo build --package rs_desktop_standalone --bin rs_desktop_standalone --release " .. standalone_feature_args)
        os.cd(engine_root_dir)
        os.exec("xmake build_android_target --mode=debug --target=aarch64-linux-android")
        os.exec("xmake build_android_target --mode=release --target=aarch64-linux-android")
        os.exec("xmake build_android_target --mode=debug --target=x86_64-linux-android")
        os.exec("xmake build_android_target --mode=release --target=x86_64-linux-android")
    end)
    set_menu {
        usage = "xmake ci",
    }
