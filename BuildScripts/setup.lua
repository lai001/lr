local rs_target_dir = rs_target_dir
local deps_dir = deps_dir
task("copy_shared_libs")
do
    local ffmpeg_dir = ffmpeg_dir
    local engine_root_dir = engine_root_dir
    local dotnet_sdk_dir = dotnet_sdk_dir
    on_run(function()
        import("core.project.config")
        import("net.http")
        import("utils.archive")
        import("lib.detect.find_program")
        import("core.project.task")
        config.load()
        local ndk_dir = config.get("ndk")

        local function setup_ffmpeg(build_type)
            local target_dir = path.join(rs_target_dir, build_type)
            os.mkdir(target_dir)
            os.cp(path.join(ffmpeg_dir, "bin/*.dll"), target_dir)
        end

        local function setup_dotnet(build_type)
            local target_dir = path.join(rs_target_dir, build_type)
            os.mkdir(target_dir)
            os.cp(path.join(dotnet_sdk_dir, "packs/Microsoft.NETCore.App.Host.win-x64/8.0.6/runtimes/win-x64/native/nethost.dll"), target_dir)
        end

        setup_ffmpeg("debug")
        setup_ffmpeg("release")
        setup_dotnet("debug")
        setup_dotnet("release")
        local arch_x86_64_dir = path.join(engine_root_dir, "Android/Template/rs_android/src/main/jniLibs/x86_64")
        local arch_arm64_v8a_dir = path.join(engine_root_dir, "Android/Template/rs_android/src/main/jniLibs/arm64-v8a")
        os.cp(path.join(deps_dir, "ffmpeg_android/arm64-v8a/lib/*.so"), arch_arm64_v8a_dir)
        os.cp(path.join(deps_dir, "ffmpeg_android/x86_64/lib/*.so"), arch_x86_64_dir)
        os.cp(path.join(ndk_dir, "toolchains/llvm/prebuilt/windows-x86_64/sysroot/usr/lib", "x86_64-linux-android", "libc++_shared.so"), arch_x86_64_dir)
        os.cp(path.join(ndk_dir, "toolchains/llvm/prebuilt/windows-x86_64/sysroot/usr/lib", "aarch64-linux-android", "libc++_shared.so"), arch_arm64_v8a_dir)
    end)
    set_menu {
        usage = "xmake copy_shared_libs",
        description = "Copy the required dynamic libraries",
        options = {
            { nil, "copy_shared_libs", nil, nil, nil },
        }
    }
end

task("create_default_load_plugins_file")
do
    on_run(function()
        os.exec(path.join(rs_target_dir, "release/rs_build_tool") .. " create-default-load-plugins-file")
    end)
    set_menu {
        usage = "xmake create_default_load_plugins_file",
        description = "Create default load plugins file",
    }
end

task("setup")
do
    on_run(function()
        os.exec("xmake download_deps")
        os.exec("xmake build_3rdparty")
        os.exec("xmake compile_tool")
        os.exec("xmake copy_shared_libs")
        os.exec("xmake gen_config")
        os.exec("xmake create_default_load_plugins_file")
    end)
    set_menu {
        usage = "xmake setup",
        description = "Initialize project",
        options = {
            { nil, "setup", nil, nil, nil },
        }
    }
end