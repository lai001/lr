set_xmakever("2.7.8")

local deps_dir = ".xmake/deps/"
local rs_project_name = "rs_computer_graphics"
local csharp_workspace_name = "ExampleApplication"
local gizmo_dir = deps_dir .. "egui-gizmo"
local quickjs_dir = deps_dir .. "quickjs"

option("enable_dotnet")
    set_default(false)
    set_showmenu(true)
option_end()

option("enable_quickjs")
    set_default(false)
    set_showmenu(true)    
option_end()

local is_enable_dotnet = get_config("enable_dotnet")
if is_enable_dotnet == nil then
    is_enable_dotnet = false
end

local is_enable_quickjs = get_config("enable_quickjs")
if is_enable_quickjs == nil then
    is_enable_quickjs = false
end

task("download_deps")
    on_run(function () 
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
            local link = "https://download.visualstudio.microsoft.com/download/pr/ca13c6f1-3107-4cf8-991c-f70edc1c1139/a9f90579d827514af05c3463bed63c22/" .. dotnetSDKFilename

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
                    git.clone("https://github.com/c-smile/quickjspp.git", {outputdir = quickjs_dir})
                else 
                    git.clone("https://github.com/bellard/quickjs.git", {outputdir = quickjs_dir})
                end
                git.checkout("master", {repodir = quickjs_dir})
            end
        end

        if os.exists(gizmo_dir) == false then
            git.clone("https://github.com/jakobhellermann/egui-gizmo.git", {outputdir = gizmo_dir})
            git.checkout("main", {repodir = gizmo_dir})
        end
    end)
    set_menu {
        usage = "xmake download_deps",
        description = "",
        options = {
            {nil, "download_deps", nil, nil, "xmake download_deps"},
        }        
    } 

task("fmt")
    on_run(function () 
        import("lib.detect.find_program")
        for _, file in ipairs(os.files("rs_computer_graphics/src/**.rs")) do
            os.execv(find_program("rustfmt"), { "--edition=2018", file })
        end
        for _, file in ipairs(os.files("rs_dotnet/src/**.rs")) do
            os.execv(find_program("rustfmt"), { "--edition=2018", file })
        end      
        for _, file in ipairs(os.files("rs_quickjs/src/**.h")) do
            os.execv(find_program("clang-format"), { "-style=microsoft", "-i", file })
        end      
        for _, file in ipairs(os.files("rs_quickjs/src/**.c")) do
            os.execv(find_program("clang-format"), { "-style=microsoft", "-i", file })
        end             
        os.execv(find_program("dotnet"), { "format", "./ExampleApplication/ExampleApplication.sln" })
    end)
    set_menu {
        usage = "xmake fmt",
        description = "",
        options = {
            {nil, "fmt", nil, nil, "xmake fmt"},
        }        
    } 

task("build_target")
    on_run(function ()
        import("lib.detect.find_program")
        import("core.base.json")
        import("core.base.option")
        import("core.project.config")
        config.load()

        local workspace = "$(scriptdir)" .. "/" .. rs_project_name
        local csharp_workspace_path = "$(scriptdir)" .. "/" .. csharp_workspace_name

        local function build(rs_build_args, csharp_build_args, mode) 
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
            local path = "rs_computer_graphics/target/" .. mode .. "/Project.json"
            local project = {
                paths = {
                    resource_dir = absolute("Resource"),
                    shader_dir = absolute("rs_computer_graphics/src/shader"),
                    intermediate_dir = "./Intermediate",
                    scripts_dir = absolute("./Scripts"),
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
            json.savefile(path, project)
        end 
        local mode = option.get("mode")
        if mode == nil then
            mode = "debug" 
        end

        is_enable_dotnet = get_config("enable_dotnet")
        is_enable_quickjs = get_config("enable_quickjs")
        local rs_build_features = {  }
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
            build(table.join({ "build", "--release" }, rs_build_features), { "build", "-c", "Release", "./" .. csharp_workspace_name ..".sln" }, mode)
        end
    end)   
    set_menu {
        usage = "xmake build_target",
        description = "",
        options = {
            {nil, "build_target", nil, nil, "xmake build_target"},
            {"m", "mode", "kv",  nil, nil },
        }        
    } 
    
task("build_clean")
    on_run(function ()
        os.tryrm(rs_project_name .. "/target")
        os.tryrm("rs_dotnet/target")
        os.tryrm("rs_quickjs/target")
        os.tryrm(csharp_workspace_name .. "/.vs")
        os.tryrm(".vscode")
        os.tryrm("build")
        for _, dir in ipairs(os.dirs(csharp_workspace_name .. "/**/obj")) do
            os.tryrm(dir) 
        end
    end)   
    set_menu {
        usage = "xmake build_clean",
        description = "",
        options = {
            {nil, "build_clean", nil, nil, "xmake build_clean"},
        }        
    } 

task("setup_project")
    on_run(function ()
        import("net.http")
        import("utils.archive")
        import("lib.detect.find_program")
        import("core.project.task")

        local function setup_dotnet(buildType) 
            local target_dir = rs_project_name .. "/target/"
            os.mkdir(target_dir .. buildType)
            local nethost = deps_dir .. "dotnetSDK/packs/Microsoft.NETCore.App.Host.win-x64/6.0.16/runtimes/win-x64/native/nethost"
            local target_nethost = target_dir .. buildType .. "/nethost"
            os.cp(nethost .. ".dll", target_nethost .. ".dll")
            os.cp(nethost .. ".lib", target_nethost .. ".lib")        
        end

        task.run("download_deps")
        if is_enable_dotnet then 
            setup_dotnet("debug")
            setup_dotnet("release")
        end
    end)
    set_menu {
        usage = "xmake setup_project",
        description = "",
        options = {
            {nil, "setup_project", nil, nil, "xmake setup_project"},
        }        
    } 

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
            add_includedirs(quickjs_dir, {public = true})
            add_includedirs("rs_quickjs/src", {public = true})
            add_defines({"CONFIG_BIGNUM", "JS_STRICT_NAN_BOXING"})
        else 
            add_files("rs_quickjs/src/*.c")
            add_headerfiles("rs_quickjs/src/*.h")

            add_files(quickjs_dir .. "/*.c")
            add_headerfiles(quickjs_dir .. "/*.h")
            remove_files(quickjs_dir .. "/run-test262.c")
            remove_files(quickjs_dir .. "/qjsc.c")
            remove_files(quickjs_dir .. "/qjs.c")
            remove_files(quickjs_dir .. "/unicode_gen.c")
            add_includedirs(quickjs_dir, {public = true})
            add_includedirs("rs_quickjs/src", {public = true})
            add_links("m", "dl", "pthread")
            add_cflags(format([[-D_GNU_SOURCE -DCONFIG_VERSION="%s" -DCONFIG_BIGNUM]], os.date('%Y-%m-%d %H:%M:%S')))
        end    
end