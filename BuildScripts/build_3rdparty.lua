local engine_root_dir = engine_root_dir
local ffmpeg_dir = ffmpeg_dir
local russimp_prebuild_dir = russimp_prebuild_dir

task("build_3rdparty")
do
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        import("core.base.option")
        import("core.project.task")
        config.load()

        os.exec("xmake f -a x64 -m debug -p windows -k static")
        os.exec("xmake build gpmetis")
        os.exec("xmake f -a x64 -m release -p windows -k static")
        os.exec("xmake build gpmetis")
        os.exec("xmake f -a arm64-v8a -m debug -p android -k static")
        os.exec("xmake build gpmetis")
        os.exec("xmake build tracy")
        os.exec("xmake f -a arm64-v8a -m release -p android -k static")
        os.exec("xmake build gpmetis")
        os.exec("xmake build tracy")
        os.exec("xmake f -a x64 -m debug -p windows -k static --enable_quickjs=y")
        os.exec("xmake build quickjs")
        os.exec("xmake f -a x64 -m release -p windows -k static --enable_quickjs=y")
        os.exec("xmake build quickjs")        
    end)
    set_menu {
        usage = "xmake build_3rdparty",
        description = "Build 3rdparty libraries",
    }
end
