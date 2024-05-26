task("setup")
do
    local ffmpeg_dir = ffmpeg_dir
    on_run(function()
        import("net.http")
        import("utils.archive")
        import("lib.detect.find_program")
        import("core.project.task")

        local function setup_ffmpeg(build_type, target)
            local target_dir = target .. "/target/"
            os.mkdir(target_dir .. build_type)
            os.cp(ffmpeg_dir .. "/bin/*.dll", target_dir .. build_type)
        end

        setup_ffmpeg("debug", "rs_editor")
        setup_ffmpeg("release", "rs_editor")
        setup_ffmpeg("debug", "rs_desktop_standalone")
        setup_ffmpeg("release", "rs_desktop_standalone")
        setup_ffmpeg("debug", "rs_media_cmd")
        setup_ffmpeg("release", "rs_media_cmd")
    end)
    set_menu {
        usage = "xmake setup",
        description = "Initialize Project",
        options = {
            { nil, "setup", nil, nil, nil },
        }
    }
end