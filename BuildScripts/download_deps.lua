local deps_dir = deps_dir
local quickjs_dir = quickjs_dir
local gizmo_dir = gizmo_dir
local ffmpeg_dir = ffmpeg_dir
local metis_dir = metis_dir
local gklib_dir = gklib_dir
local russimp_prebuild_dir = russimp_prebuild_dir
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
            local dotnetSDKFilename = "dotnet-sdk-6.0.408-win-x64.zip"
            local link =
                "https://download.visualstudio.microsoft.com/download/pr/ca13c6f1-3107-4cf8-991c-f70edc1c1139/a9f90579d827514af05c3463bed63c22/" ..
                dotnetSDKFilename

            if os.exists(deps_dir .. dotnetSDKFilename) == false then
                http.download(link, deps_dir .. dotnetSDKFilename)
            end

            if os.exists(deps_dir .. "dotnetSDK") == false and os.exists(deps_dir .. dotnetSDKFilename) then
                archive.extract(deps_dir .. dotnetSDKFilename, deps_dir .. "dotnetSDK")
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
            git.clone("https://github.com/urholaukkarinen/egui-gizmo.git", { outputdir = gizmo_dir })
            git.checkout("45a4afb8900d7f46c50d2c35db7cf5a79cc23466", { repodir = gizmo_dir })
        end

        if os.exists("Resource/Remote/neon_photostudio_2k.exr") == false then
            local link = "https://dl.polyhaven.org/file/ph-assets/HDRIs/exr/2k/neon_photostudio_2k.exr"
            http.download(link, "Resource/Remote/neon_photostudio_2k.exr")
        end

        local ffmpeg_zip_filename = deps_dir .. "ffmpeg-n6.0-31-g1ebb0e43f9-win64-gpl-shared-6.0.zip"
        if os.exists(ffmpeg_zip_filename) == false then
            local link =
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2023-07-24-12-50/ffmpeg-n6.0-31-g1ebb0e43f9-win64-gpl-shared-6.0.zip"
            http.download(link, ffmpeg_zip_filename)
        end
        if os.exists(ffmpeg_zip_filename) and os.exists(ffmpeg_dir) == false then
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

        local meshopt_rs_dir = deps_dir .. "meshopt-rs"
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

        local russimp_file = deps_dir .. "russimp-2.0.1-x86_64-pc-windows-msvc-static.tar.gz"
        if os.exists(russimp_file) == false then
            local link = "https://github.com/jkvargas/russimp-sys/releases/download/v2.0.1/russimp-2.0.1-x86_64-pc-windows-msvc-static.tar.gz"
            http.download(link, russimp_file)
        end
        if os.exists(russimp_prebuild_dir) == false and os.exists(russimp_file) then
            archive.extract(russimp_file, russimp_prebuild_dir)
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
