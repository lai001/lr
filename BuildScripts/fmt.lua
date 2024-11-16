local last_str = ''
local function io_overwrite(io, str)
   io.write(('\b \b'):rep(#last_str))
   io.write(str)
   io.flush()
   last_str = str
end

task("fmt") do
    on_run(function()
        import("lib.detect.find_program")
        local rs_projects = table.join(os.dirs("rs_*"), os.dirs("programs/rs_*"), os.dirs("crates/rs_*"))
        local rustfmt_args = { "--edition=2018" }
        for _, project in ipairs(rs_projects) do
            for _, file in ipairs(os.files(project .. "/src/**.rs")) do
                table.insert(rustfmt_args, file)
            end
        end
        local clang_format_args = { "-style=microsoft", "-i" }
        for _, file in ipairs(os.files("rs_quickjs/src/**.h")) do
            table.insert(clang_format_args, file)
        end
        for _, file in ipairs(os.files("rs_quickjs/src/**.c")) do
            table.insert(clang_format_args, file)
        end
        io_overwrite(io, "Formatting *.rs files, 0%")
        os.execv(find_program("rustfmt"), rustfmt_args)
        io_overwrite(io, "Formatting *.c,*.h files, 33%")
        os.execv(find_program("clang-format"), clang_format_args)
        io_overwrite(io, "Formatting *.cs files, 66%")
        os.execv(find_program("dotnet"), { "format", "./ExampleApplication/ExampleApplication.sln" })
        io_overwrite(io, "Formatting, 100%")
    end)
    set_menu {
        usage = "xmake fmt",
        description = "Format code",
        options = {
            { nil, "fmt", nil, nil, nil },
        }
    }
end