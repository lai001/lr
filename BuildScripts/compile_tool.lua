local engine_root_dir = engine_root_dir

task("compile_tool")
do
    on_run(function()
        os.cd(path.join(engine_root_dir, "rs_build_tool"))
        os.exec("cargo build --package rs_build_tool --bin rs_build_tool")
        os.exec("cargo build --package rs_build_tool --bin rs_build_tool --release")
        os.cd(path.join(engine_root_dir, "rs_shader_compiler_lsp"))
        os.exec("cargo build --package rs_shader_compiler_lsp --bin rs_shader_compiler_lsp")
        os.exec("cargo build --package rs_shader_compiler_lsp --bin rs_shader_compiler_lsp --release")
        os.cd(path.join(engine_root_dir, "rs_media_cmd"))
        os.exec("cargo build --package rs_media_cmd --bin rs_media_cmd")
        os.exec("cargo build --package rs_media_cmd --bin rs_media_cmd --release")                
    end)
    set_menu {
        usage = "xmake compile_tool",
    }
end