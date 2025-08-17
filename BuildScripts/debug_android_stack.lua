local engine_root_dir = engine_root_dir
task("debug_android_stack")
    on_run(function()
        import("core.project.config")
        import("core.base.option")
        config.load()
        local input_file_path = option.get("input")
        local symbol_file_path = option.get("symbol")
        if input_file_path == nil or symbol_file_path == nil then
            return
        end
        local ndk_dir = config.get("ndk")
        local addr2line = path.join(ndk_dir, "toolchains/llvm/prebuilt/windows-x86_64/bin/llvm-addr2line.exe")
        local log = io.readfile(input_file_path)
        if log:find("%*%*%* %*%*%* %*%*%* %*%*%* %*%*%* %*%*%* %*%*%* %*%*%*") then
            for addr in log:gmatch("pc%s+(%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x)") do
                local cm = format("%s -e %s %s", addr2line, symbol_file_path, addr)
                os.exec(cm)
            end
        else
            print("No crash marker found.")
        end
    end)
    set_menu {
        usage = "xmake debug_android_stack",
        description = "Debug android stack",
        options = {
            { "i", "input", "kv", nil, "Input file", nil },
            { "s", "symbol", "kv", nil, "Symbol file", nil }
        }
    }