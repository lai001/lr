task("fmt")
    on_run(function()
        import("lib.detect.find_program")
        local rs_projects = os.dirs("rs_*")
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
        os.execv(find_program("rustfmt"), rustfmt_args)
        os.execv(find_program("clang-format"), clang_format_args)
        os.execv(find_program("dotnet"), { "format", "./ExampleApplication/ExampleApplication.sln" })
    end)
    set_menu {
        usage = "xmake fmt",
        description = "Format code",
        options = {
            { nil, "fmt", nil, nil, nil },
        }
    }
task_end()