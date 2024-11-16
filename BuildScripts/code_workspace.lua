
local rs_project_name = rs_project_name
local ffmpeg_dir = ffmpeg_dir
local russimp_prebuild_dir = russimp_prebuild_dir

local function reflection_generator_workspace_file(json)
    local extraArgs = { "--release" }
    local media_cmd = {
        ["folders"] = { {
            ["path"] = path.absolute("./")
        } },
        ["settings"] = {
            ["rust-analyzer.linkedProjects"] = {
                path.absolute("./programs/rs_reflection_generator/Cargo.toml"),
            },
            ["rust-analyzer.runnables.extraArgs"] = extraArgs
        }
    }
    json.savefile(path.join(path.absolute("./"), "media_cmd.code-workspace"), media_cmd)
end

local function media_cmd_workspace_file(json)
    local extraEnv = {
        ["FFMPEG_DIR"] = ffmpeg_dir,
    }
    local media_cmd = {
        ["folders"] = { {
            ["path"] = path.absolute("./")
        } },
        ["settings"] = {
            ["rust-analyzer.linkedProjects"] = {
                path.absolute("./rs_media/Cargo.toml"),
                path.absolute("./rs_media_cmd/Cargo.toml")
            },
            ["rust-analyzer.cargo.extraEnv"] = extraEnv,
            ["rust-analyzer.server.extraEnv"] = extraEnv,
            ["rust-analyzer.check.extraEnv"] = extraEnv,
            ["rust-analyzer.runnables.extraEnv"] = extraEnv
        }
    }
    json.savefile(path.join(path.absolute("./"), "media_cmd.code-workspace"), media_cmd)
end

local function proc_macros_test_workspace_file(json)
    local proc_macros_test = {
        ["folders"] = { {
            ["path"] = path.absolute("./")
        } },
        ["settings"] = {
            ["rust-analyzer.linkedProjects"] = {
                path.absolute("./rs_proc_macros/Cargo.toml"),
                path.absolute("./rs_proc_macros_test/Cargo.toml")
            },
        }
    }
    json.savefile(path.join(path.absolute("./"), "proc_macros_test.code-workspace"), proc_macros_test)
end

local function audio_workspace_file(json)
    local extraEnv = {
        ["FFMPEG_DIR"] = ffmpeg_dir,
    }
    local audio = {
        ["folders"] = { {
            ["path"] = path.absolute("./")
        } },
        ["settings"] = {
            ["rust-analyzer.linkedProjects"] = {
                path.absolute("./rs_audio/Cargo.toml"),
            },
            ["rust-analyzer.cargo.extraEnv"] = extraEnv,
            ["rust-analyzer.server.extraEnv"] = extraEnv,
            ["rust-analyzer.check.extraEnv"] = extraEnv,
            ["rust-analyzer.runnables.extraEnv"] = extraEnv,
            ["rust-analyzer.runnables.extraTestBinaryArgs"] = {
                "--show-output",
                "--nocapture"
            },
        }
    }
    json.savefile(path.join(path.absolute("./"), "audio.code-workspace"), audio)
end

local function build_tool_workspace_file(json)
    local audio = {
        ["folders"] = { {
            ["path"] = path.absolute("./")
        } },
        ["settings"] = {
            ["rust-analyzer.linkedProjects"] = {
                path.absolute("./rs_build_tool/Cargo.toml"),
            },
        }
    }
    json.savefile(path.join(path.absolute("./"), "build_tool.code-workspace"), audio)
end

task("code_workspace") do
    on_run(function(in_plat, in_target, in_mode, in_launch)
        import("core.project.config")
        import("core.base.option")
        import("core.base.json")
        config.load()
        local is_enable_debug_refcell = false
        local is_enable_dotnet = get_config("enable_dotnet")
        is_enable_dotnet = (is_enable_dotnet and {is_enable_dotnet} or {false})[1]

        local is_enable_quickjs = get_config("enable_quickjs")
        is_enable_quickjs = (is_enable_quickjs and {is_enable_quickjs} or {false})[1]

        local plat = (in_plat and {in_plat} or {option.get("plat")})[1]
        plat = (plat and {plat} or {"windows"})[1]

        local target = (in_target and {in_target} or {option.get("target")})[1]
        target = (target and {target} or {})[1]

        local mode = (in_mode and {in_mode} or {option.get("mode")})[1]
        mode = (mode and {mode} or {"debug"})[1]

        local launch_type = (in_launch and {in_launch} or {option.get("launch")})[1]
        launch_type = (launch_type and {launch_type} or {"editor"})[1]

        local is_enable_renderdoc = option.get("renderdoc")
        is_enable_renderdoc = (is_enable_renderdoc and {is_enable_renderdoc} or {false})[1]

        local features = {}
        local extraArgs = {}
        if mode == "release" then
            table.join2(extraArgs, "--release")
        end
        if is_enable_debug_refcell then
            table.join2(extraArgs, "--target=x86_64-pc-windows-msvc")
            table.join2(extraArgs, "-Zbuild-std")
            table.join2(extraArgs, "-Zbuild-std-features=debug_refcell")
        end
        local linkedProjects = {}

        if plat == "android" then
            table.join2(linkedProjects, path.absolute("./rs_android/Cargo.toml"))
        elseif plat == "windows" then
            if launch_type == "editor" then
                table.join2(linkedProjects, path.absolute("./rs_editor/Cargo.toml"))
            elseif launch_type == "standalone" then
                table.join2(linkedProjects, path.absolute("./rs_desktop_standalone/Cargo.toml"))
            end
        end
        local associations = {}
        local extraEnv = {
            ["FFMPEG_DIR"] = ffmpeg_dir,
            ["RUSSIMP_PACKAGE_DIR"] = russimp_prebuild_dir
        }

        if launch_type == "editor" then
            table.join2(features, launch_type)
        elseif launch_type == "standalone" then
            table.join2(features, launch_type)
        end
        if is_enable_renderdoc then
            table.join2(features, "renderdoc")
        end
        table.join2(features, "plugin_shared_crate")
        -- table.join2(features, "plugin_dotnet")
        -- table.join2(features, "plugin_v8")

        if #features == 0 then
            features = nil
        end
        if #extraArgs == 0 then
            extraArgs = nil
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
                ["rust-analyzer.runnables.extraEnv"] = extraEnv,
                ["rust-analyzer.checkOnSave"] = false
            }
        }
        local file_name = format("%s_%s_%s.code-workspace", launch_type, plat, target)
        local save_path = path.join(path.absolute("./"), file_name)
        print(save_path)
        json.savefile(save_path, code_workspace)

        proc_macros_test_workspace_file(json)
        media_cmd_workspace_file(json)
        audio_workspace_file(json)
        build_tool_workspace_file(json)
        reflection_generator_workspace_file(json)
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
                " - release" },
            { "l", "launch", "kv", "editor", "Set launch type.",
                " - editor",
                " - standalone" },
            { nil, "renderdoc", "k", nil, "Enable renderdoc feature." }
        }
    }
end