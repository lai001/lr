local engine_root_dir = engine_root_dir

local function get_last_part(s)
    local part = s:match(".*/(.*)")
    return part or s
end

local function build_program(os_module, mode_arg, program_root_path)
    local os = os_module
    local name = get_last_part(program_root_path)
    os.cd(path.join(engine_root_dir, program_root_path))
    os.exec(format("cargo build --package %s --bin %s %s", name, name, mode_arg))
end

task("compile_tool")
do
    on_run(function()
        import("core.base.option")
        local mode = option.get("mode")
        local mode_arg = ""
        if mode == "release" then
            mode_arg = "--release"
        end
        build_program(os, mode_arg, "rs_build_tool")
        build_program(os, mode_arg, "rs_shader_compiler_lsp")
        build_program(os, mode_arg, "rs_media_cmd")
        build_program(os, mode_arg, "programs/rs_reflection_generator")
        build_program(os, mode_arg, "programs/rs_v8_binding_api_generator")
        build_program(os, mode_arg, "rs_shader_compiler")
    end)
    set_menu {
        usage = "xmake compile_tool",
        description = "Compile tool",
        options = {
            { "m", "mode", "kv", "release", "Set the build mode.",
                " - debug",
                " - release" }
        }
    }
end
