
local rs_project_name = rs_project_name
local ffmpeg_dir = ffmpeg_dir
local russimp_prebuild_dir = russimp_prebuild_dir
task("code_workspace") do
    on_run(function(in_plat, in_target, in_mode)
        import("core.project.config")
        import("core.base.option")
        import("core.base.json")
        config.load()
        local is_enable_dotnet = get_config("enable_dotnet")
        local is_enable_quickjs = get_config("enable_quickjs")
        local features = {}
        local extraArgs = {}
        local linkedProjects = { 
            path.absolute("./rs_render/Cargo.toml") ,
            path.absolute("./rs_foundation/Cargo.toml") ,
            path.absolute("./rs_engine/Cargo.toml") ,
            path.absolute("./rs_artifact/Cargo.toml") ,
        }
        local plat = (in_plat and {in_plat} or {option.get("plat")})[1]
        plat = (plat and {plat} or {"windows"})[1]
        
        if plat == "android" then
            table.join2(linkedProjects, path.absolute("./rs_android/Cargo.toml"))
        elseif plat == "windows" then
            table.join2(linkedProjects, path.absolute("./rs_computer_graphics/Cargo.toml"))
            table.join2(linkedProjects, path.absolute("./rs_editor/Cargo.toml"))
            table.join2(linkedProjects, path.absolute("./rs_hotreload_plugin/Cargo.toml"))
            table.join2(linkedProjects, path.absolute("./rs_desktop_standalone/Cargo.toml"))
        end
        local associations = {}
        local extraEnv = {
            ["FFMPEG_DIR"] = ffmpeg_dir,
            ["RUSSIMP_PACKAGE_DIR"] = russimp_prebuild_dir
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
        local target = (in_target and {in_target} or {option.get("target")})[1]
        target = (target and {target} or {})[1]
   
        local mode = (in_mode and {in_mode} or {option.get("mode")})[1]
        mode = (mode and {mode} or {"debug"})[1]
        if mode == "release" then
            table.join2(extraArgs, "--release")
        end

        local code_workspace = {
            ["folders"] = { {
                ["path"] = path.absolute("./")
            } },
            ["settings"] = {
                ["rust-analyzer.cargo.features"] = features,
                ["rust-analyzer.linkedProjects"] = linkedProjects,
                ["rust-analyzer.cargo.target"] = target,
                ["rust-analyzer.runnables.extraArgs"] = extraArgs,
                ["files.associations"] = associations,
                ["rust-analyzer.cargo.extraEnv"] = extraEnv,
                ["rust-analyzer.server.extraEnv"] = extraEnv,
                ["rust-analyzer.check.extraEnv"] = extraEnv,
                ["rust-analyzer.runnableEnv"] = extraEnv
            }
        }
        local file_name = format("%s_%s.code-workspace", rs_project_name, plat)
        local save_path = path.join(path.absolute("./"), file_name)
        print(save_path)
        json.savefile(save_path, code_workspace)
    end)
    set_menu {
        usage = "xmake code_workspace",
        description = "Generate vscode project workspace file.",
        options = {
            { "t", "target", "kv", nil, "Set target.",
                " - aarch64-linux-android",
                " - armv7-linux-androideabi",
                " - x86_64-linux-android",
                " - i686-linux-android",
                " - arm-linux-androideabi",
                " - x86_64-pc-windows-msvc" },
            { "p", "plat", "kv", "windows", "Set platfrom.",
                " - windows",
                " - android" },
            { "m", "mode", "kv", "debug", "Set build configuration.",
                " - debug",
                " - release" }                
        }
    }
end