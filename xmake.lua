set_xmakever("2.7.8")

deps_dir = ".xmake/deps/"
rs_project_name = "rs_computer_graphics"
csharp_workspace_name = "ExampleApplication"
gizmo_dir = deps_dir .. "egui-gizmo"
quickjs_dir = deps_dir .. "quickjs"
metis_dir = path.absolute(deps_dir .. "METIS")
gklib_dir = deps_dir .. "GKlib"
ffmpeg_dir = path.absolute(deps_dir .. "ffmpeg-n6.0-31-g1ebb0e43f9-win64-gpl-shared-6.0")
russimp_prebuild_dir = path.absolute(deps_dir)
engine_root_dir = path.absolute("./")
tracy_root_dir = path.absolute(deps_dir .. "tracy")
dotnet_sdk_dir = path.absolute(deps_dir .. "dotnetSDK")

includes("BuildScripts/gen_config.lua")
includes("BuildScripts/build_android_target.lua")
includes("BuildScripts/code_workspace.lua")
includes("BuildScripts/fmt.lua")
includes("BuildScripts/download_deps.lua")
includes("BuildScripts/build_clean.lua")
includes("BuildScripts/install_editor.lua")
includes("BuildScripts/ci.lua")
includes("BuildScripts/setup.lua")
includes("BuildScripts/build_3rdparty.lua")
includes("BuildScripts/compile_build_tool.lua")

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

local function ter_op(condition, true_value, false_value)
    return (condition and {true_value} or {false_value})[1]
end

local function create_project(buildir, plat, arch, mode, is_shipping, path_module)
    local absolute = path_module.absolute
    local project = {
        paths = {
            resource_dir = ter_op(is_shipping, "./Resource", absolute("Resource")),
            shader_dir = ter_op(is_shipping, "./shader", absolute("rs_computer_graphics/src/shader")),
            intermediate_dir = "./Intermediate",
            scripts_dir = ter_op(is_shipping, "./Scripts", absolute("./Scripts")),
            gpmetis_program_path = ter_op(is_shipping, "./gpmetis.exe", absolute(format("%s/%s/%s/%s/gpmetis.exe", buildir, plat, arch, mode))),
        },
        dotnet = {
            config_path = "./ExampleApplication.runtimeconfig.json",
            assembly_path = "./ExampleApplication.dll",
            type_name = "ExampleApplication.Entry, ExampleApplication",
            method_name = "Main",
        },
        user_script = {
            path = "./tmp/UserScript.dll"
        }
    }
    return project
end

local is_enable_dotnet = get_config_default("enable_dotnet", false)
local is_enable_quickjs = get_config_default("enable_quickjs", false)

task("build_target") do
    local rs_project_name = rs_project_name
    local csharp_workspace_name = csharp_workspace_name
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

        local function create_project_json(mode, path_module)
            os.cd("$(scriptdir)")
            local target_path = path_module.absolute("rs_computer_graphics/target/" .. mode)
            local project = create_project(get_config("buildir"), get_config("plat"), get_config("arch"), mode, false, path_module)
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
            create_project_json("debug", path)
            build(table.join({ "build" }, rs_build_features), { "build", "./" .. csharp_workspace_name .. ".sln" }, mode)
        elseif mode == "release" then
            create_project_json("release", path)
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
end

task("setup_project") do
    local rs_project_name = rs_project_name
    local ffmpeg_dir = ffmpeg_dir
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
        task.run("code_workspace", {}, "windows", "x86_64-pc-windows-msvc")
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
end

task("install_target")
    on_run(function()
        import("core.base.option")
        import("core.base.json")
        import("core.project.config")
        config.load()
        local function install_files(build_type, path_module)
            local source_dir = "./rs_computer_graphics/target/" .. build_type
            local install_dir = path_module.join(get_config("buildir"), get_config("plat"), "bin", build_type)
            os.mkdir(install_dir)
            os.trycp(source_dir .. "/*.dll", install_dir)
            os.trycp(source_dir .. "/Project.json", install_dir)
            os.trycp(source_dir .. "/*.exe", install_dir)
            os.trycp("Resource", install_dir)
            os.trycp("Scripts", install_dir)
            os.trycp("rs_computer_graphics/src/shader", install_dir)
            os.trycp(path_module.join(get_config("buildir"), get_config("plat"), get_config("arch"), build_type, "gpmetis.exe"), install_dir)
            local project = create_project(get_config("buildir"), get_config("plat"), get_config("arch"), mode, true, path_module)
            json.savefile(install_dir .. "/Project.json", project)
        end
        local mode = option.get("mode")
        if mode == nil then
            mode = "debug"
        end
        if mode == "debug" then
            install_files("debug", path)
        elseif mode == "release" then
            install_files("release", path)
        end
    end)
    set_menu {
        usage = "xmake install_target",
        description = "Install target.",
        options = {
            { "m", "mode", "kv", "debug", "Set the install mode.", " - debug", " - release" },
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
        if is_plat("android") then
            add_defines("MAX_PATH=255")
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

target("tracy")
    set_languages("cxx11")
    set_kind("$(kind)")
    set_basename("tracy-client")
    add_rules("mode.debug", "mode.release")
    add_defines("TRACY_ENABLE")
    add_files(tracy_root_dir .. "/public/TracyClient.cpp")