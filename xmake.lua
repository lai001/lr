set_xmakever("2.7.8")

local deps_dir = ".xmake/deps/"
local rs_project_name = "rs_computer_graphics"
local csharp_workspace_name = "ExampleApplication"
local gizmo_dir = deps_dir .. "egui-gizmo"

task("download_deps")
    on_run(function () 
        import("net.http")
        import("utils.archive")
        import("devel.git")
        os.mkdir(deps_dir)
        local dotnetSDKFilename = "dotnet-sdk-6.0.408-win-x64.zip"
        local link = "https://download.visualstudio.microsoft.com/download/pr/ca13c6f1-3107-4cf8-991c-f70edc1c1139/a9f90579d827514af05c3463bed63c22/" .. dotnetSDKFilename

        if os.exists(deps_dir .. dotnetSDKFilename) == false then
            http.download(link, deps_dir .. dotnetSDKFilename)
        end

        if os.exists(deps_dir .. "dotnetSDK") == false and os.exists(deps_dir .. dotnetSDKFilename) then
            archive.extract(deps_dir .. dotnetSDKFilename, deps_dir .. "dotnetSDK")
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

        local workspace = "$(scriptdir)" .. "/" .. rs_project_name
        local csharp_workspace_path = "$(scriptdir)" .. "/" .. csharp_workspace_name

        local function build(rs_build_args, csharp_build_args) 
            os.cd(workspace)
            os.execv(find_program("cargo"), rs_build_args)
            os.cd(csharp_workspace_path)
            os.execv(find_program("dotnet"), csharp_build_args)
        end

        local function create_project_json(mode, absolute)
            os.cd("$(scriptdir)")
            local path = "rs_computer_graphics/target/" .. mode .. "/Project.json"
            local project = {
                paths = {
                    resource_dir = absolute("Resource"),
                    shader_dir = absolute("rs_computer_graphics/src/shader"),
                    intermediate_dir = "./Intermediate",
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
        if mode == "debug" then
            create_project_json("debug", path.absolute)
            build({ "build" }, { "build", "./" .. csharp_workspace_name .. ".sln" })
        elseif mode == "release" then
            create_project_json("release", path.absolute)
            build({ "build", "--release" }, { "build", "-c", "Release", "./" .. csharp_workspace_name ..".sln" })
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
        os.tryrm(csharp_workspace_name .. "/.vs")
        os.tryrm(".vscode")
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

        local function setup(buildType) 
            local target_dir = rs_project_name .. "/target/"
            os.mkdir(target_dir .. buildType)
            local nethost = deps_dir .. "dotnetSDK/packs/Microsoft.NETCore.App.Host.win-x64/6.0.16/runtimes/win-x64/native/nethost"
            local target_nethost = target_dir .. buildType .. "/nethost"
            os.cp(nethost .. ".dll", target_nethost .. ".dll")
            os.cp(nethost .. ".lib", target_nethost .. ".lib")        
        end

        task.run("download_deps")
        setup("debug")
        setup("release")
    end)
    set_menu {
        usage = "xmake setup_project",
        description = "",
        options = {
            {nil, "setup_project", nil, nil, "xmake setup_project"},
        }        
    } 