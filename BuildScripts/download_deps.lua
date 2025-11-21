local deps_dir = deps_dir
local quickjs_dir = quickjs_dir
local gizmo_dir = gizmo_dir
local ffmpeg_dir = ffmpeg_dir
local metis_dir = metis_dir
local gklib_dir = gklib_dir
local russimp_prebuild_dir = russimp_prebuild_dir
local tracy_root_dir = tracy_root_dir
local dotnet_sdk_dir = dotnet_sdk_dir
local check_hash_files = check_hash_files
local hash_files = hash_files
local kcp_root_dir = kcp_root_dir

task("hash_files")
do
    on_run(function()
        import("core.base.option")
        local input = option.get("input")
        local is_trace = option.get("trace") ~= nil
        if input == nil then
            raise()
        end
        local value = hash_files(os, io, is_trace, input)
        print(value)
    end)
    set_menu {
        usage = "xmake hash_files",
        description = "Hash files",
        options = {
            { "i", "input", "kv", nil, "File path or pattern match" },
            { "d", "trace", nil, nil, "Enable trace mode" },
        }
    }
end

task("download_deps")
do
    on_run(function()
        import("net.http")
        import("utils.archive")
        import("devel.git")
        import("core.project.config")
        config.load()
        os.mkdir(deps_dir)

        local is_enable_dotnet = get_config("enable_dotnet")
        local is_enable_quickjs = get_config("enable_quickjs")

        if is_enable_dotnet then
            local dotnetSDKFilename = "dotnet-sdk-8.0.302-win-x64.zip"
            local link =
                "https://download.visualstudio.microsoft.com/download/pr/5af098e1-e433-4fda-84af-3f54fd27c108/6bd1c6e48e64e64871957289023ca590/" ..
                dotnetSDKFilename

            if not check_hash_files(os, io, path.join(deps_dir, dotnetSDKFilename), "ddb7dc983df37b20bd03b57c27c1ecdc3eee8f7187cca9d6498023381f912dc0") then
                http.download(link, path.join(deps_dir, dotnetSDKFilename))
            end
            if not check_hash_files(os, io, dotnet_sdk_dir .. "/**/*.exe", "fa916994a7eedcc3050f9bda118dfc4fb05a85bb1a04ed344fd8a79274e1b7f4") then
                archive.extract(path.join(deps_dir, dotnetSDKFilename), dotnet_sdk_dir)
            end
        end

        if is_enable_quickjs then
            if os.exists(quickjs_dir) == false then
                if is_plat("windows") then
                    git.clone("https://github.com/c-smile/quickjspp.git", { outputdir = quickjs_dir })
                else
                    git.clone("https://github.com/bellard/quickjs.git", { outputdir = quickjs_dir })
                end
                git.checkout("master", { repodir = quickjs_dir })
            end
        end

        if os.exists(gizmo_dir) == false then
            git.clone("https://github.com/urholaukkarinen/transform-gizmo.git", { outputdir = gizmo_dir })
            git.checkout("00be178c38a09a6a8df2ae4f557b7a12fcdafe14", { repodir = gizmo_dir })
        end

        if not check_hash_files(os, io, "Resource/Remote/neon_photostudio_2k.exr", "da6a36f6fa031c3896c915eea7e2794d62fdf77a2b800085ca276b1962381e15") then
            local link = "https://dl.polyhaven.org/file/ph-assets/HDRIs/exr/2k/neon_photostudio_2k.exr"
            http.download(link, "Resource/Remote/neon_photostudio_2k.exr")
        end

        local ffmpeg_zip_filename = path.join(deps_dir, "ffmpeg-n6.0-31-g1ebb0e43f9-win64-gpl-shared-6.0.zip")
        if not check_hash_files(os, io, ffmpeg_zip_filename, "ee396b019b29624dd2e9a2b538ed74e9a9b36eb9a60868d89efc255c80accbab") then
            local link =
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2023-07-24-12-50/ffmpeg-n6.0-31-g1ebb0e43f9-win64-gpl-shared-6.0.zip"
            http.download(link, ffmpeg_zip_filename)
        end
        if not check_hash_files(os, io, ffmpeg_dir .. "/**/*.dll", "9d2d55a148c1579ff929f79cf6b47ae96bdee670139dba88f424629422605a87") then
            archive.extract(ffmpeg_zip_filename, deps_dir)
        end

        if os.exists("Resource/Remote/BigBuckBunny.mp4") == false then
            local link = "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4"
            http.download(link, "Resource/Remote/BigBuckBunny.mp4")
        end

        if os.exists("Resource/Remote/sample-15s.mp3") == false then
            local link = "https://download.samplelib.com/mp3/sample-15s.mp3"
            http.download(link, "Resource/Remote/sample-15s.mp3")
        end

        local meshopt_rs_dir = path.join(deps_dir, "meshopt-rs")
        if os.exists(meshopt_rs_dir) == false then
            git.clone("https://github.com/gwihlidal/meshopt-rs.git", { outputdir = meshopt_rs_dir })
            git.checkout("master", { repodir = meshopt_rs_dir })
        end

        if os.exists(metis_dir) == false then
            git.clone("https://github.com/KarypisLab/METIS.git", { outputdir = metis_dir })
            git.checkout("v5.2.1", { repodir = metis_dir })
        end

        if os.exists(gklib_dir) == false then
            git.clone("https://github.com/KarypisLab/GKlib.git", { outputdir = gklib_dir })
            git.checkout("master", { repodir = gklib_dir })
        end

        local russimp_file = path.join(deps_dir, "russimp-2.0.2-x86_64-pc-windows-msvc-static.tar.gz")
        if os.exists(russimp_file) == false then
            local link = "https://github.com/jkvargas/russimp-sys/releases/download/v2.0.2/russimp-2.0.2-x86_64-pc-windows-msvc-static.tar.gz"
            http.download(link, russimp_file)
        end
        -- if os.exists(russimp_prebuild_dir) == false and os.exists(russimp_file) then
        --     archive.extract(russimp_file, russimp_prebuild_dir)
        -- end

        local tracy_archive_file = path.join(deps_dir, "Tracy-0.13.0.zip")
        local tracy_file = path.join(deps_dir, "Tracy-0.13.0")
        if os.exists(tracy_archive_file) == false then
            local link = "https://github.com/wolfpld/tracy/releases/download/v0.13.0/windows-0.13.0.zip"
            http.download(link, tracy_archive_file)
        end
        if os.exists(tracy_file) == false and os.exists(tracy_archive_file) then
            archive.extract(tracy_archive_file, tracy_file)
        end

        if os.exists(tracy_root_dir) == false then
            git.clone("https://github.com/wolfpld/tracy.git", { outputdir = tracy_root_dir })
            git.checkout("v0.13.0", { repodir = tracy_root_dir })
        end

        if os.exists(kcp_root_dir) == false then
            git.clone("https://github.com/skywind3000/kcp.git", { outputdir = kcp_root_dir })
            git.checkout("f4f3a89cc632647dabdcb146932d2afd5591e62e", { repodir = kcp_root_dir })
        end
    end)
    set_menu {
        usage = "xmake download_deps",
        description = "Download dependencies.",
        options = {
            { nil, "download_deps", nil, nil, nil },
        }
    }
end
