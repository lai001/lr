task("build_android_target")
do
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        import("core.project.task")
        config.load()
        local mode = option.get("mode")
        if mode == nil then
            mode = "debug"
        end
        local target = "aarch64-linux-android"
        if option.get("target") ~= nil then
            target = option.get("target")
        end
        local project_name = "rs_android"
        local old = os.cd(project_name)

        local jobs = option.get("jobs")
        if jobs == nil then
            jobs = os.meminfo().availsize//2000
        end
        if mode == "debug" then
            os.exec("cargo build --target %s -j %d", target, jobs)
        else
            os.exec("cargo build --target %s -r -j %d", target, jobs)
        end
        os.cd(old)
        local target_map = { }
        target_map["aarch64-linux-android"] = "arm64-v8a"
        target_map["armv7-linux-androideabi"] = "armeabi-v7a"
        target_map["x86_64-linux-android"] = "x86_64"
        target_map["i686-linux-android"] = "x86"
        target_map["arm-linux-androideabi"] = "armeabi"
        local arch = target_map[target]
        local function cp_print(src, target)
            print("Copying %s to %s", src, target)
            os.cp(src, target)
        end
        if mode == "debug" then
            cp_print(format("%s/target/%s/debug/lib%s.so", project_name, target, project_name),
                format("Android/Template/%s/src/main/jniLibs/%s/lib%s.so", project_name, arch, project_name))
        else
            cp_print(format("%s/target/%s/release/lib%s.so", project_name, target, project_name),
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
end
