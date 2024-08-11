task("cargo_upgrade_all")
do
    on_run(function()
        local black_list = { 
            ["rs_computer_graphics"] = true
        }
        for _, dir in ipairs(os.dirs("rs_*")) do
            if black_list[dir] == nil then
                local old = os.cd(dir)
                os.exec("cargo upgrade")
                os.cd(old)
            end
        end
    end)
    set_menu {
        usage = "xmake cargo_upgrade_all",
        description = "Upgrade dependency version requirements in Cargo.toml manifest files.",
        options = {
            { nil, "cargo_upgrade_all", nil, nil, nil },
        }
    }
end
