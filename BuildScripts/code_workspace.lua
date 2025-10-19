
local rs_project_name = rs_project_name
local ffmpeg_dir = ffmpeg_dir
local russimp_prebuild_dir = russimp_prebuild_dir

local function is_valid_table(t)
    if t == nil then
        return false
    end
    for _, _ in pairs(t) do
        return true
    end
    return false
end

local function write_workspace_file(json, context)
    local contents = {}
    if is_valid_table(context.folders) then
        contents["folders"] = {}
        for _, folder in ipairs(context.folders) do
            table.join2(contents["folders"], {{["path"] = folder}})
        end
    end
    contents["settings"] = {}
    if is_valid_table(context.linked_projects) then
        local linked_projects = {}
        for i, linked_project in ipairs(context.linked_projects) do
            linked_projects[i] = path.join(linked_project, "Cargo.toml")
        end
        contents["settings"]["rust-analyzer.linkedProjects"] = linked_projects
    end
    if is_valid_table(context.features) then
        contents["settings"]["rust-analyzer.cargo.features"] = context.features
    end
    if context.target ~= nil and #context.target ~= 0 then
        contents["settings"]["rust-analyzer.cargo.target"] = context.target
    end
    if is_valid_table(context.runnables_extra_args) then
        contents["settings"]["rust-analyzer.runnables.extraArgs"] = context.runnables_extra_args
    end
    if is_valid_table(context.extra_env) then
        contents["settings"]["rust-analyzer.cargo.extraEnv"] = context.extra_env
        contents["settings"]["rust-analyzer.server.extraEnv"] = context.extra_env
        contents["settings"]["rust-analyzer.check.extraEnv"] = context.extra_env
        contents["settings"]["rust-analyzer.runnables.extraEnv"] = context.extra_env
    end
    if is_valid_table(context.extra_test_binary_args) then
        contents["settings"]["rust-analyzer.runnables.extraTestBinaryArgs"] = context.extra_test_binary_args
    end
    contents["settings"]["rust-analyzer.checkOnSave"] = false
    contents["settings"]["rust-analyzer.showSyntaxTree"] = false
    if context.file_stem ~= nil then
        local output_path = path.join(path.absolute("./build"), context.file_stem .. ".code-workspace")
        json.savefile(output_path, contents)
    end
end

task("code_workspace")
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
        target = (target and {target} or {""})[1]

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
            table.join2(linkedProjects, path.absolute("./rs_android"))
        elseif plat == "windows" then
            if launch_type == "editor" then
                table.join2(linkedProjects, path.absolute("./rs_editor"))
            elseif launch_type == "standalone" then
                table.join2(linkedProjects, path.absolute("./rs_desktop_standalone"))
            end
        end

        local extraEnv = {
            ["FFMPEG_DIR"] = ffmpeg_dir,
            ["RUSSIMP_PACKAGE_DIR"] = russimp_prebuild_dir
        }
        if launch_type == "standalone" and plat == "android" then
            local ndk_path = (get_config("ndk") and { get_config("ndk") } or { os.getenv("NDK_HOME") })[1]
            ndk_path = (ndk_path and { ndk_path } or { os.getenv("NDK_ROOT") })[1]
            if ndk_path == nil then
                os.raise("NDK not found")
            end
            local host = (get_config("host") and { get_config("host") } or { "windows" })[1]
            extraEnv["TARGET_CC"] = path.join(ndk_path, format("toolchains/llvm/prebuilt/%s-x86_64/bin/aarch64-linux-android30-clang.cmd", host))
            extraEnv["TARGET_CXX"] = path.join(ndk_path, format("toolchains/llvm/prebuilt/%s-x86_64/bin/aarch64-linux-android30-clang++.cmd", host))
        end

        if launch_type == "editor" then
            table.join2(features, launch_type)
        elseif launch_type == "standalone" then
            table.join2(features, launch_type)
        end
        if is_enable_renderdoc then
            table.join2(features, "renderdoc")
        end
        table.join2(features, "plugin_shared_crate")
        table.join2(features, "reflection")
        table.join2(features, "network")
        -- table.join2(features, "profiler")
        -- table.join2(features, "detect_encoding")
        -- table.join2(features, "exit_check")
        -- table.join2(features, "plugin_dotnet")
        -- table.join2(features, "plugin_v8")

        write_workspace_file(json, {
            file_stem = format("%s_%s_%s", launch_type, plat, target),
            folders = {path.absolute("./")},
            linked_projects = linkedProjects,
            runnables_extra_args = extraArgs,
            features = features,
            target = target,
            extra_env = extraEnv
        })
        write_workspace_file(json, {
            file_stem = "rs_v8_binding_api_generator",
            folders = {path.absolute("./")},
            linked_projects = {path.absolute("./programs/rs_v8_binding_api_generator")},
            runnables_extra_args = {"--release"}
        })
        write_workspace_file(json, {
            file_stem = "reflection_generator",
            folders = {path.absolute("./")},
            linked_projects = {path.absolute("./programs/rs_reflection_generator")},
            runnables_extra_args = {"--release"}
        })
        write_workspace_file(json, {
            file_stem = "build_tool",
            folders = {path.absolute("./")},
            linked_projects = {path.absolute("./rs_build_tool")}
        })
        write_workspace_file(json, {
            file_stem = "audio",
            folders = {path.absolute("./")},
            linked_projects = {path.absolute("./rs_audio")},
            extra_env = {["FFMPEG_DIR"] = ffmpeg_dir},
            extra_test_binary_args = {"--show-output", "--nocapture"}
        })
        write_workspace_file(json, {
            file_stem = "proc_macros",
            folders = {path.absolute("./")},
            linked_projects = {path.absolute("./rs_proc_macros"), path.absolute("./rs_proc_macros_test")}
        })
        write_workspace_file(json, {
            file_stem = "media_cmd",
            folders = {path.absolute("./")},
            linked_projects = {path.absolute("./rs_media"), path.absolute("./rs_media_cmd")},
            extra_env = {["FFMPEG_DIR"] = ffmpeg_dir}
        })
        write_workspace_file(json, {
            file_stem = "rs_shader_compiler",
            folders = {path.absolute("./"), path.absolute("./rs_render/shaders")},
            linked_projects = {path.absolute("./rs_shader_compiler")}
        })
        write_workspace_file(json, {
            file_stem = "network",
            folders = {path.absolute("./")},
            linked_projects = {path.absolute("./crates/rs_network")},
            extra_test_binary_args = {"--show-output", "--nocapture"}
        })
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