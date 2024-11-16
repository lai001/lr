local rs_target_dir = rs_target_dir
task("setup")
do
    local ffmpeg_dir = ffmpeg_dir
    local engine_root_dir = engine_root_dir
    local dotnet_sdk_dir = dotnet_sdk_dir
    on_run(function()
        import("net.http")
        import("utils.archive")
        import("lib.detect.find_program")
        import("core.project.task")

        local function setup_ffmpeg(build_type)
            local target_dir = path.join(rs_target_dir, build_type)
            os.mkdir(target_dir)
            os.cp(ffmpeg_dir .. "/bin/*.dll", target_dir)
        end

        local function setup_dotnet(build_type)
            local target_dir = path.join(rs_target_dir, build_type)
            os.mkdir(target_dir)
            os.cp(dotnet_sdk_dir .. "/packs/Microsoft.NETCore.App.Host.win-x64/8.0.6/runtimes/win-x64/native/nethost.dll", target_dir)
        end

        setup_ffmpeg("debug")
        setup_ffmpeg("release")
        setup_dotnet("debug")
        setup_dotnet("release")
        os.cp(path.join(ffmpeg_dir, "lib/*.so"), path.join(engine_root_dir, "Android/Template/rs_android/src/main/jniLibs/arm64-v8a"))
    end)
    set_menu {
        usage = "xmake setup",
        description = "Initialize Project",
        options = {
            { nil, "setup", nil, nil, nil },
        }
    }
end