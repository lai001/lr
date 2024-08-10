local engine_root_dir = engine_root_dir

task("compile_build_tool")
do
    on_run(function()
        os.cd(path.join(engine_root_dir, "rs_build_tool"))
        os.exec("cargo build --package rs_build_tool --bin rs_build_tool")
        os.exec("cargo build --package rs_build_tool --bin rs_build_tool --release")
    end)
    set_menu {
        usage = "xmake compile_build_tool",
    }
end
