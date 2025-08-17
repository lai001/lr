local deps_dir = deps_dir
local engine_root_dir = engine_root_dir
task("build_android_target")
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        import("core.project.task")
        config.load()
        local target_map = {}
        target_map["aarch64-linux-android"] = "arm64-v8a"
        target_map["armv7-linux-androideabi"] = "armeabi-v7a"
        target_map["x86_64-linux-android"] = "x86_64"
        target_map["i686-linux-android"] = "x86"
        target_map["arm-linux-androideabi"] = "armeabi"
        local mode = option.get("mode")
        if mode == nil then
            mode = "debug"
        end
        local target = "aarch64-linux-android"
        if option.get("target") ~= nil then
            target = option.get("target")
        end
        local jobs = option.get("jobs")
        if jobs == nil then
            jobs = os.meminfo().availsize//2000
        end
        local arch = target_map[target]
        local features = {"standalone", "plugin_shared_crate"}
        local features_arg = ""
        local is_support_profiler = arch == "arm64-v8a"
        local is_use_profiler = is_support_profiler and false
        if is_use_profiler then
            table.join2(features, "profiler")
        end
        if mode == "debug" then
            table.join2(features, "panic_hook")
        end
        for _, feature in ipairs(features) do
            features_arg = format("%s%s,", features_arg, feature)
        end
        local ffmpeg_dir = path.join(deps_dir, "ffmpeg_android", arch)
        local extra_envs = {
            ["FFMPEG_DIR"] = ffmpeg_dir,
            ["TRACY_CLIENT_LIB_PATH"] = path.join(engine_root_dir, format("build/android/%s/%s", arch, mode)),
            ["TRACY_CLIENT_LIB"] = "tracy-client",
            ["TRACY_CLIENT_STATIC"] = 1
        }
        local project_name = "rs_android"
        local old = os.cd(path.join(engine_root_dir, project_name))
        os.addenvs(extra_envs)
        if mode == "debug" then
            os.exec("cargo build --features %s --target %s -j %d", features_arg, target, jobs)
        else
            os.exec("cargo build --features %s --target %s -r -j %d", features_arg, target, jobs)
        end
        os.cd(old)
        local function cp_print(src, target)
            print("Copying %s to %s", src, target)
            os.cp(src, target)
        end
        if mode == "debug" then
            cp_print(format("build/target/%s/debug/lib%s.so", target, project_name),
                format("Android/Template/%s/src/main/jniLibs/%s/lib%s.so", project_name, arch, project_name))
        else
            cp_print(format("build/target/%s/release/lib%s.so", target, project_name),
                format("Android/Template/%s/src/main/jniLibs/%s/lib%s.so", project_name, arch, project_name))
        end
    end)
    set_menu {
        usage = "xmake build_android_target",
        description = "Build android target",
        options = {
            { "m", "mode", "kv", "debug", "Set the build mode.",
                " - debug",
                " - release" },
            { "j", "jobs", "kv", nil, "Number of parallel jobs.",
                " - <N>",
                " - release" },
            { "t", "target", "kv", "aarch64-linux-android", "Set target.",
                " - aarch64-linux-android",
                " - armv7-linux-androideabi",
                " - x86_64-linux-android",
                " - i686-linux-android",
                " - arm-linux-androideabi", },
        }
    }
