local engine_root_dir = engine_root_dir
local ffmpeg_dir = ffmpeg_dir
local russimp_prebuild_dir = russimp_prebuild_dir

task("build_3rdparty")
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        import("core.project.task")
        config.load()

        local function build(target_name, platforms, additional)
            if platforms["windows"] then
                os.exec("xmake f -a x64 -m debug -p windows -k static " .. additional)
                os.exec("xmake build " .. target_name)
                os.exec("xmake f -a x64 -m release -p windows -k static " .. additional)
                os.exec("xmake build " .. target_name)
            end
            if platforms["android"] then
                os.exec("xmake f -a arm64-v8a -m debug -p android -k static " .. additional)
                os.exec("xmake build " .. target_name)
                os.exec("xmake f -a arm64-v8a -m release -p android -k static " .. additional)
                os.exec("xmake build " .. target_name)
                os.exec("xmake f -a x86_64 -m debug -p android -k static " .. additional)
                os.exec("xmake build " .. target_name)
                os.exec("xmake f -a x86_64 -m release -p android -k static " .. additional)
                os.exec("xmake build " .. target_name)                
            end
        end
        build("gpmetis", {windows=true, android=true}, "")
        build("tracy", {windows=false, android=true}, "")
        build("quickjs", {windows=true, android=false}, "--enable_quickjs=y")
        build("kcp", {windows=true, android=true}, "")
    end)
    set_menu {
        usage = "xmake build_3rdparty",
        description = "Build 3rdparty libraries",
    }
