local engine_root_dir = engine_root_dir
local ffmpeg_dir = ffmpeg_dir
local assimp_root_dir = assimp_root_dir

task("build_assimp")
    on_run(function()
        import("lib.detect.find_program")
        local program = find_program("cmake")
        local target_dir = path.join(assimp_root_dir, "build")
        os.exec(format([[%s -S "%s" -B "%s" -DBUILD_SHARED_LIBS=OFF -DASSIMP_BUILD_ASSIMP_TOOLS=OFF -DASSIMP_BUILD_TESTS=OFF -DASSIMP_BUILD_TESTS=OFF -DASSIMP_BUILD_SAMPLES=OFF -DASSIMP_INSTALL=ON -DASSIMP_INSTALL_PDB=ON -DUSE_STATIC_CRT=OFF -DASSIMP_BUILD_ASSIMP_VIEW=OFF -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="%s"]], program, assimp_root_dir, target_dir, target_dir))
        os.exec(format("%s --build \"%s\" --config Release --target install", program, target_dir))
    end)
    set_menu {
        usage = "xmake build_assimp",
        description = "Build assimp library",
    }

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
        os.exec("xmake build_assimp")
    end)
    set_menu {
        usage = "xmake build_3rdparty",
        description = "Build 3rdparty libraries",
    }
