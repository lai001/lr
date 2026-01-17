local engine_root_dir = engine_root_dir
local ffmpeg_dir = ffmpeg_dir
local russimp_prebuild_dir = russimp_prebuild_dir

local function build_feature_args(features)
    local features_arg = ""
    for _, feature in ipairs(features) do
        features_arg = format("%s%s,", features_arg, feature)
    end
    return format("--features %s", features_arg)
end

task("build_editor_target")
    on_run(function()
        import("core.base.option")
        local editor_feature_args = build_feature_args({
            "editor", "renderdoc", "plugin_shared_crate", "plugin_dotnet", "plugin_v8", "network"
        })
        local mode = option.get("mode")
        if mode == nil then
            mode = ""
        end
        local mode_arg = ""
        if mode == "release" then
            mode_arg = "--release"
        end
        os.cd(path.join(engine_root_dir, "rs_editor"))
        os.exec(format("cargo build --package rs_editor --bin editor %s %s", mode_arg, editor_feature_args))
    end)
    set_menu {
        usage = "xmake build_editor_target",
        options = {
            { "m", "mode", "kv", "debug", "Set the build mode.",
                " - debug",
                " - release" }
        }
    }

task("build_desktop_standalone_target")
    on_run(function()
        import("core.base.option")
        local standalone_feature_args = build_feature_args({
            "standalone", "plugin_shared_crate", "plugin_v8", "network"
        })
        local mode = option.get("mode")
        if mode == nil then
            mode = "debug"
        end
        local mode_arg = ""
        if mode == "release" then
            mode_arg = "--release"
        end
        os.cd(path.join(engine_root_dir, "rs_desktop_standalone"))
        os.exec(format("cargo build --package rs_desktop_standalone --bin rs_desktop_standalone %s %s", mode_arg, standalone_feature_args))
    end)
    set_menu {
        usage = "xmake build_editor_target",
        options = {
            { "m", "mode", "kv", "debug", "Set the build mode.",
                " - debug",
                " - release" }
        }
    }

task("ci")
    on_run(function()
        import("core.base.option")
        local mode = option.get("mode")
        if mode == nil then
            mode = "debug"
        end
        os.cd(path.join(engine_root_dir, "build/target/release"))
        os.exec("./rs_shader_compiler.exe")
        os.exec(format("xmake build_editor_target --mode=%s", mode))
        os.exec(format("xmake build_desktop_standalone_target --mode=%s", mode))
        os.cd(engine_root_dir)
        local targets = {"aarch64-linux-android"}
        targets[#targets + 1] = "x86_64-linux-android"
        for _, target in ipairs(targets) do
            os.exec(format("xmake build_android_target --mode=%s --target=%s", mode, target))
        end
    end)
    set_menu {
        usage = "xmake ci",
        options = {
            { "m", "mode", "kv", "debug", "Set the build mode.",
                " - debug",
                " - release" }
        }
    }
