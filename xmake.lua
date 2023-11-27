set_xmakever("2.7.8")

local deps_dir = ".xmake/deps/"
local rs_project_name = "rs_computer_graphics"
local csharp_workspace_name = "ExampleApplication"
local gizmo_dir = deps_dir .. "egui-gizmo"
local quickjs_dir = deps_dir .. "quickjs"
local metis_dir = path.absolute(deps_dir .. "METIS")
local gklib_dir = deps_dir .. "GKlib"
local ffmpeg_dir = path.absolute(deps_dir .. "ffmpeg-n6.0-31-g1ebb0e43f9-win64-gpl-shared-6.0")

option("enable_dotnet")
    set_default(false)
    set_showmenu(true)
option_end()

option("enable_quickjs")
    set_default(false)
    set_showmenu(true)
option_end()

local function get_config_default(name, default_value) 
    local cfg_value = get_config(name)
    if cfg_value == nil then
        cfg_value = default_value
    end
    return cfg_value
end

local is_enable_dotnet = get_config_default("enable_dotnet", false)
local is_enable_quickjs = get_config_default("enable_quickjs", false)

task("code_workspace")
    on_run(function()
        import("core.project.config")
        import("core.base.json")
        config.load()
        is_enable_dotnet = get_config("enable_dotnet")
        is_enable_quickjs = get_config("enable_quickjs")
        local features = { "" }
        local linkedProjects = { path.absolute("./rs_computer_graphics/Cargo.toml") }
        local associations = {}
        local ffmpeg_env = {
            ["FFMPEG_DIR"] = ffmpeg_dir
        }

        if is_enable_quickjs then
            table.join2(features, "rs_quickjs")
            table.join2(linkedProjects, path.absolute("./rs_quickjs/Cargo.toml"))
            table.join2(associations, { ["quickjs.h"] = "c" })
        end
        if is_enable_dotnet then
            table.join2(features, "rs_dotnet")
            table.join2(linkedProjects, path.absolute("./rs_dotnet/Cargo.toml"))
        end

        local code_workspace = {
            ["folders"] = { {
                ["path"] = path.absolute("./")
            } },
            ["settings"] = {
                ["rust-analyzer.cargo.features"] = features,
                ["rust-analyzer.linkedProjects"] = linkedProjects,
                ["files.associations"] = associations,
                ["rust-analyzer.cargo.extraEnv"] = ffmpeg_env,
                ["rust-analyzer.server.extraEnv"] = ffmpeg_env,
                ["rust-analyzer.check.extraEnv"] = ffmpeg_env,
                ["rust-analyzer.runnableEnv"] = ffmpeg_env
            }
        }
        json.savefile("./" .. rs_project_name .. ".code-workspace", code_workspace)
    end)
    set_menu {
        usage = "xmake code_workspace",
        description = "Generate vscode project workspace file.",
        options = {
            { nil, "code_workspace", nil, nil, nil },
        }
    }
task_end()

task("download_deps")
    on_run(function()
        import("net.http")
        import("utils.archive")
        import("devel.git")
        import("core.project.config")
        config.load()
        os.mkdir(deps_dir)

        is_enable_dotnet = get_config("enable_dotnet")
        is_enable_quickjs = get_config("enable_quickjs")

        if is_enable_dotnet then
            local dotnetSDKFilename = "dotnet-sdk-6.0.408-win-x64.zip"
            local link =
            "https://download.visualstudio.microsoft.com/download/pr/ca13c6f1-3107-4cf8-991c-f70edc1c1139/a9f90579d827514af05c3463bed63c22/" ..
            dotnetSDKFilename

            if os.exists(deps_dir .. dotnetSDKFilename) == false then
                http.download(link, deps_dir .. dotnetSDKFilename)
            end

            if os.exists(deps_dir .. "dotnetSDK") == false and os.exists(deps_dir .. dotnetSDKFilename) then
                archive.extract(deps_dir .. dotnetSDKFilename, deps_dir .. "dotnetSDK")
            end
        end

        if is_enable_quickjs then
            if os.exists(quickjs_dir) == false then
                if is_plat("windows") then
                    git.clone("https://github.com/c-smile/quickjspp.git", { outputdir = quickjs_dir })
                else
                    git.clone("https://github.com/bellard/quickjs.git", { outputdir = quickjs_dir })
                end
                git.checkout("master", { repodir = quickjs_dir })
            end
        end

        if os.exists(gizmo_dir) == false then
            git.clone("https://github.com/jakobhellermann/egui-gizmo.git", { outputdir = gizmo_dir })
            git.checkout("main", { repodir = gizmo_dir })
        end

        if os.exists("Resource/Remote/neon_photostudio_2k.exr") == false then
            local link = "https://dl.polyhaven.org/file/ph-assets/HDRIs/exr/2k/neon_photostudio_2k.exr"
            http.download(link, "Resource/Remote/neon_photostudio_2k.exr")
        end

        local ffmpeg_zip_filename = deps_dir .. "ffmpeg-n6.0-31-g1ebb0e43f9-win64-gpl-shared-6.0.zip"
        if os.exists(ffmpeg_zip_filename) == false then
            local link =
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2023-07-24-12-50/ffmpeg-n6.0-31-g1ebb0e43f9-win64-gpl-shared-6.0.zip"
            http.download(link, ffmpeg_zip_filename)
        end
        if os.exists(ffmpeg_zip_filename) and os.exists(ffmpeg_dir) == false then
            archive.extract(ffmpeg_zip_filename, deps_dir)
        end

        if os.exists("Resource/Remote/BigBuckBunny.mp4") == false then
            local link = "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4"
            http.download(link, "Resource/Remote/BigBuckBunny.mp4")
        end

        if os.exists("Resource/Remote/sample-15s.mp3") == false then
            local link = "https://download.samplelib.com/mp3/sample-15s.mp3"
            http.download(link, "Resource/Remote/sample-15s.mp3")
        end

        local meshopt_rs_dir = deps_dir .. "meshopt-rs"
        if os.exists(meshopt_rs_dir) == false then
            git.clone("https://github.com/gwihlidal/meshopt-rs.git", { outputdir = meshopt_rs_dir })
            git.checkout("master", { repodir = meshopt_rs_dir })
        end

        if os.exists(metis_dir) == false then
            git.clone("https://github.com/KarypisLab/METIS.git", { outputdir = metis_dir })
            git.checkout("v5.2.1", { repodir = metis_dir })
        end

        if os.exists(gklib_dir) == false then
            git.clone("https://github.com/KarypisLab/GKlib.git", { outputdir = gklib_dir })
            git.checkout("master", { repodir = gklib_dir })
        end        
    end)
    set_menu {
        usage = "xmake download_deps",
        description = "Download dependencies.",
        options = {
            { nil, "download_deps", nil, nil, nil },
        }
    }
task_end()

task("fmt")
    on_run(function()
        import("lib.detect.find_program")
        local rs_projects = { "rs_computer_graphics", "rs_dotnet", "rs_media", "rs_quickjs", "rs_foundation", "rs_metis" }
        local rustfmt_args = { "--edition=2018" }
        for _, project in ipairs(rs_projects) do
            for _, file in ipairs(os.files(project .. "/src/**.rs")) do
                table.insert(rustfmt_args, file)
            end
        end
        local clang_format_args = { "-style=microsoft", "-i" }
        for _, file in ipairs(os.files("rs_quickjs/src/**.h")) do
            table.insert(clang_format_args, file)
        end
        for _, file in ipairs(os.files("rs_quickjs/src/**.c")) do
            table.insert(clang_format_args, file)
        end
        os.execv(find_program("rustfmt"), rustfmt_args)
        os.execv(find_program("clang-format"), clang_format_args)
        os.execv(find_program("dotnet"), { "format", "./ExampleApplication/ExampleApplication.sln" })
    end)
    set_menu {
        usage = "xmake fmt",
        description = "Format code",
        options = {
            { nil, "fmt", nil, nil, nil },
        }
    }
task_end()

task("build_target")
    on_run(function()
        import("lib.detect.find_program")
        import("core.base.json")
        import("core.base.option")
        import("core.project.config")
        config.load()

        os.addenvs({ FFMPEG_DIR = ffmpeg_dir })

        local workspace = "$(scriptdir)" .. "/" .. rs_project_name
        local csharp_workspace_path = "$(scriptdir)" .. "/" .. csharp_workspace_name

        local function build(rs_build_args, csharp_build_args, mode)
            os.execv(find_program("xmake"), { "f", "-m", mode })
            os.execv(find_program("xmake"), { "build", "gpmetis" })
            if is_enable_quickjs then
                os.cd("$(scriptdir)")
                if mode == "debug" then
                    os.execv(find_program("xmake"), { "f", "-m", "debug", "--enable_quickjs=y" })
                elseif mode == "release" then
                    os.execv(find_program("xmake"), { "f", "-m", "release", "--enable_quickjs=y" })
                end
                os.execv(find_program("xmake"), { "build", "quickjs" })
            end
            os.cd(workspace)
            os.execv(find_program("cargo"), rs_build_args)
            if is_enable_dotnet then
                os.cd(csharp_workspace_path)
                os.execv(find_program("dotnet"), csharp_build_args)
            end
        end

        local function create_project_json(mode, absolute)
            os.cd("$(scriptdir)")
            local target_path = absolute("rs_computer_graphics/target/" .. mode)
            local project = {
                paths = {
                    resource_dir = absolute("Resource"),
                    shader_dir = absolute("rs_computer_graphics/src/shader"),
                    intermediate_dir = target_path .. "/Intermediate",
                    scripts_dir = absolute("./Scripts"),
                    gpmetis_program_path = absolute(format("%s/%s/%s/%s/gpmetis.exe", get_config("buildir"), get_config("plat"), get_config("arch"), get_config("mode"))),
                },
                dotnet = {
                    config_path = target_path .. "/ExampleApplication.runtimeconfig.json",
                    assembly_path = target_path .. "/ExampleApplication.dll",
                    type_name = "ExampleApplication.Entry, ExampleApplication",
                    method_name = "Main",
                },
                user_script = {
                    path = target_path .. "/tmp/UserScript.dll"
                }
            }
            json.savefile(target_path .. "/Project.json", project)
        end
        local mode = option.get("mode")
        if mode == nil then
            mode = "debug"
        end

        is_enable_dotnet = get_config("enable_dotnet")
        is_enable_quickjs = get_config("enable_quickjs")
        local rs_build_features = {}
        if is_enable_dotnet or is_enable_quickjs then
            table.join2(rs_build_features, { "--features" })
        end
        if is_enable_dotnet then
            table.join2(rs_build_features, { "rs_dotnet" })
        end
        if is_enable_quickjs then
            table.join2(rs_build_features, { "rs_quickjs" })
        end
        if mode == "debug" then
            create_project_json("debug", path.absolute)
            build(table.join({ "build" }, rs_build_features), { "build", "./" .. csharp_workspace_name .. ".sln" }, mode)
        elseif mode == "release" then
            create_project_json("release", path.absolute)
            build(table.join({ "build", "--release" }, rs_build_features),
                { "build", "-c", "Release", "./" .. csharp_workspace_name .. ".sln" }, mode)
        end
    end)
    set_menu {
        usage = "xmake build_target [mode]",
        description = "Build target.",
        options = {
            { "m", "mode", "kv", "debug", "Set the build mode.", " - debug", " - release" },
        }
    }
task_end()

task("build_clean")
    on_run(function()
        os.tryrm(rs_project_name .. "/target")
        os.tryrm("rs_dotnet/target")
        os.tryrm("rs_quickjs/target")
        os.tryrm("rs_media/target")
        os.tryrm("rs_foundation/target")
        os.tryrm(csharp_workspace_name .. "/.vs")
        os.tryrm(".vscode")
        os.tryrm("build")
        for _, dir in ipairs(os.dirs(csharp_workspace_name .. "/**/obj")) do
            os.tryrm(dir)
        end
    end)
    set_menu {
        usage = "xmake build_clean",
        description = "Clean up build files.",
        options = {
            { nil, "build_clean", nil, nil, nil },
        }
    }
task_end()

task("setup_project")
    on_run(function()
        import("net.http")
        import("utils.archive")
        import("lib.detect.find_program")
        import("core.project.task")

        local function setup_dotnet(buildType)
            local target_dir = rs_project_name .. "/target/"
            os.mkdir(target_dir .. buildType)
            local nethost = deps_dir ..
            "dotnetSDK/packs/Microsoft.NETCore.App.Host.win-x64/6.0.16/runtimes/win-x64/native/nethost"
            local target_nethost = target_dir .. buildType .. "/nethost"
            os.cp(nethost .. ".dll", target_nethost .. ".dll")
            os.cp(nethost .. ".lib", target_nethost .. ".lib")
        end
        local function setup_ffmpeg(buildType)
            local target_dir = rs_project_name .. "/target/"
            os.mkdir(target_dir .. buildType)
            os.cp(ffmpeg_dir .. "/bin/*.dll", target_dir .. buildType)
        end

        task.run("download_deps")
        task.run("code_workspace")
        if is_enable_dotnet then
            setup_dotnet("debug")
            setup_dotnet("release")
        end
        setup_ffmpeg("debug")
        setup_ffmpeg("release")
    end)
    set_menu {
        usage = "xmake setup_project",
        description = "Initialize Project",
        options = {
            { nil, "setup_project", nil, nil, nil },
        }
    }
task_end()

if is_enable_quickjs then
    target("quickjs")
        set_kind("$(kind)")
        add_languages("c11")
        add_rules("mode.debug", "mode.release")
        if is_plat("windows") then
            local source_files = {
                "cutils.c",
                "libregexp.c",
                "libunicode.c",
                "quickjs.c",
                "quickjs-libc.c",
                "libbf.c",
            }
            local header_files = {
                "cutils.h",
                "libregexp.h",
                "libregexp-opcode.h",
                "libunicode.h",
                "libunicode-table.h",
                "list.h",
                "quickjs.h",
                "quickjs-atom.h",
                "quickjs-libc.h",
                "quickjs-opcode.h",
                "quickjs-jsx.h",
            }
            add_files("rs_quickjs/src/*.c")
            add_headerfiles("rs_quickjs/src/*.h")

            for i, v in ipairs(source_files) do
                add_files(path.join(quickjs_dir, v))
            end
            for i, v in ipairs(header_files) do
                add_headerfiles(path.join(quickjs_dir, v))
            end
            add_includedirs(quickjs_dir, { public = true })
            add_includedirs("rs_quickjs/src", { public = true })
            add_defines({ "CONFIG_BIGNUM", "JS_STRICT_NAN_BOXING" })
        else
            add_files("rs_quickjs/src/*.c")
            add_headerfiles("rs_quickjs/src/*.h")

            add_files(quickjs_dir .. "/*.c")
            add_headerfiles(quickjs_dir .. "/*.h")
            remove_files(quickjs_dir .. "/run-test262.c")
            remove_files(quickjs_dir .. "/qjsc.c")
            remove_files(quickjs_dir .. "/qjs.c")
            remove_files(quickjs_dir .. "/unicode_gen.c")
            add_includedirs(quickjs_dir, { public = true })
            add_includedirs("rs_quickjs/src", { public = true })
            add_links("m", "dl", "pthread")
            add_cflags(format([[-D_GNU_SOURCE -DCONFIG_VERSION="%s" -DCONFIG_BIGNUM]], os.date('%Y-%m-%d %H:%M:%S')))
        end
end

function gklib_add_defines()
    add_defines("USE_GKREGEX")
    add_defines("IDXTYPEWIDTH=32")
    add_defines("REALTYPEWIDTH=32")
    if is_plat("windows") then
        add_defines("__thread=__declspec(thread)")
        add_defines("MSC")
        add_defines("WIN32")
        add_defines("_CRT_SECURE_NO_DEPRECATE")
    end
    if is_mode("debug") then
        add_defines("DEBUG")
    else
        add_defines("NDEBUG")
    end
end

function create_metis_program(target_name, source_files, source_files2)
    target(target_name)
        set_languages("c11")
        add_rules("mode.debug", "mode.release")
        for _, file in ipairs(source_files) do
            add_files(metis_dir .. "/programs/" .. file)
        end
        if source_files ~= nil then
            add_files(source_files2)
        end
        add_deps("GKlib")
        add_deps("metis")
        gklib_add_defines()
        add_includedirs(metis_dir .. "/libmetis")
    target_end()
end

target("GKlib")
    set_kind("$(kind)")
    set_languages("c11")
    add_rules("mode.debug", "mode.release")
    add_files(gklib_dir .. "/*.c")
    add_headerfiles(gklib_dir .. "/*.h")
    add_includedirs(gklib_dir, { public = true })
    if is_plat("windows") then 
        add_headerfiles(gklib_dir .. "/win32/*.h")
        add_includedirs(gklib_dir .. "/win32", { public = true })
        add_files(gklib_dir .. "/win32/*.c")
    end
    gklib_add_defines()

target("metis")
    set_kind("$(kind)")
    set_languages("c11")
    add_rules("mode.debug", "mode.release")
    add_files(metis_dir .. "/libmetis/*.c")
    add_headerfiles(metis_dir .. "/libmetis/*.h")
    add_includedirs(metis_dir .. "/include", { public = true })  
    add_deps("GKlib")
    gklib_add_defines()

target("gpmetis")
    set_languages("c11")
    add_rules("mode.debug", "mode.release")
    local c_files = { "gpmetis.c", "cmdline_gpmetis.c", "io.c", "stat.c" }
    for _, file in ipairs(c_files) do
        add_files(metis_dir .. "/programs/" .. file)
    end
    add_deps("GKlib")
    add_deps("metis")
    gklib_add_defines()
    add_includedirs(metis_dir .. "/libmetis")  

create_metis_program("gpmetis", { "gpmetis.c", "cmdline_gpmetis.c", "io.c", "stat.c" })
-- create_metis_program("ndmetis", { "ndmetis.c", "cmdline_ndmetis.c", "io.c", "smbfactor.c" })
-- create_metis_program("mpmetis", { "mpmetis.c", "cmdline_mpmetis.c", "io.c", "stat.c" })
-- create_metis_program("m2gmetis", { "m2gmetis.c", "cmdline_m2gmetis.c", "io.c" })
-- create_metis_program("graphchk", { "graphchk.c", "io.c" })
-- create_metis_program("cmpfillin", { "cmpfillin.c", "io.c", "smbfactor.c" })
-- create_metis_program("metis_test", {}, { metis_dir .. "/test/mtest.c" })
