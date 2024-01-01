
local rs_project_name = rs_project_name
local ffmpeg_dir = ffmpeg_dir
task("code_workspace") do
    on_run(function()
        import("core.project.config")
        import("core.base.option")
        import("core.base.json")
        config.load()
        local is_enable_dotnet = get_config("enable_dotnet")
        local is_enable_quickjs = get_config("enable_quickjs")
        local features = { }
        local linkedProjects = { 
            path.absolute("./rs_computer_graphics/Cargo.toml") ,
            path.absolute("./rs_editor/Cargo.toml") ,
            path.absolute("./rs_render/Cargo.toml") ,
            path.absolute("./rs_foundation/Cargo.toml") ,
            path.absolute("./rs_engine/Cargo.toml") ,
            path.absolute("./rs_artifact/Cargo.toml") ,
        }
        if get_config("plat") == "android" then
            table.join2(linkedProjects, path.absolute("./rs_android/Cargo.toml"))
        end
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
        local target = nil
        if option.get("target") ~= nil then
            target = option.get("target")
        end
        local code_workspace = {
            ["folders"] = { {
                ["path"] = path.absolute("./")
            } },
            ["settings"] = {
                ["rust-analyzer.cargo.features"] = features,
                ["rust-analyzer.linkedProjects"] = linkedProjects,
                ["rust-analyzer.cargo.target"] = target,
                ["rust-analyzer.runnables.extraArgs"] = {},
                ["files.associations"] = associations,
                ["rust-analyzer.cargo.extraEnv"] = ffmpeg_env,
                ["rust-analyzer.server.extraEnv"] = ffmpeg_env,
                ["rust-analyzer.check.extraEnv"] = ffmpeg_env,
                ["rust-analyzer.runnableEnv"] = ffmpeg_env
            }
        }
        local save_path = path.join(path.absolute("./"), rs_project_name .. ".code-workspace")
        print(save_path)
        json.savefile(save_path, code_workspace)
    end)
    set_menu {
        usage = "xmake code_workspace",
        description = "Generate vscode project workspace file.",
        options = {
            { nil, "target", "kv", nil, "Set target.", " - aarch64-linux-android" },
        }
    }
end