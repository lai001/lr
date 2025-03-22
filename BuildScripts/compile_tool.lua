local engine_root_dir = engine_root_dir

local function get_last_part(s)
    local part = s:match(".*/(.*)")
    return part or s
end

local function build_program(os_module, program_root_path)
    local os = os_module
    local name = get_last_part(program_root_path)
    os.cd(path.join(engine_root_dir, program_root_path))
    os.exec(format("cargo build --package %s --bin %s", name, name))
    os.exec(format("cargo build --package %s --bin %s --release", name, name))
end

task("compile_tool")
do
    on_run(function()
        build_program(os, "rs_build_tool")
        build_program(os, "rs_shader_compiler_lsp")
        build_program(os, "rs_media_cmd")
        build_program(os, "programs/rs_reflection_generator")
        build_program(os, "programs/rs_v8_binding_api_generator")
        build_program(os, "rs_shader_compiler")
    end)
    set_menu {
        usage = "xmake compile_tool",
        description = "Compile tool",
    }
end
