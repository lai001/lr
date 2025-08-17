local engine_root_dir = engine_root_dir
task("generate_jni_funtions")
    on_run(function()
        import("lib.detect.find_program")
        local envs = os.getenvs()
        local java_home = envs["JAVA_HOME"]
        local android_sdk_home = envs["ANDROID_SDK_HOME"]
        local search_paths = {}
        table.join2(search_paths, java_home)
        if java_home ~= nil then
            table.join2(search_paths, path.join(java_home, "bin"))
        end
        local javac_program = find_program("javac", {paths = search_paths, check = "-version"})
        local tmp_dir = path.join(engine_root_dir, "build/java/jni")
        for _, java_file in ipairs(os.files("Android/Template/libraries/lrjni/src/main/java/com/lai001/lib/lrjni/*.java")) do
            if os.exists(tmp_dir) == false then
                os.mkdir(tmp_dir)
            end
            java_file = path.absolute(java_file)
            local class_output_dir = tmp_dir
            local header_output_dir = tmp_dir
            local class_paths = {
                path.join(android_sdk_home, "platforms/android-34/android.jar")
            }
            local class_paths_arg = ""
            for _, class_path in ipairs(class_paths) do
                class_paths_arg = format([[%s "%s"]], class_paths_arg, class_path)
            end
            local cmd = format([["%s" --class-path "%s" -d "%s" -h "%s" "%s"]], javac_program, class_paths_arg, class_output_dir, header_output_dir, java_file)
            print(cmd)
            os.exec(cmd)
        end
        local param_map = {}
        param_map["JNIEnv *"]="jni::JNIEnv"
        param_map["jobject"]="jni::sys::jobject"
        param_map["jlong"]="jni::sys::jlong"
        param_map["jstring"]="jni::sys::jstring"
        param_map["jint"]="jni::sys::jint"
        param_map["jdouble"]="jni::sys::jdouble"
        param_map["jfloat"]="jni::sys::jfloat"
        param_map["jchar"]="jni::sys::jchar"
        param_map["jbyte"]="jni::sys::jbyte"
        param_map["jboolean"]="jni::sys::jboolean"
        param_map["jclass"]="jni::objects::JClass"
        for _, header_file in ipairs(os.files(path.join(tmp_dir, "*.h"))) do
            local basenaem = path.basename(header_file)
            local output_file = path.join(path.directory(header_file), basenaem .. ".rs")
            local data = io.readfile(header_file)
            local contents = ""
            for match in data:gmatch("JNIEXPORT(.-);") do
                for return_type, func_name, params in match:gmatch("(%w+)%s+%w+%s+(%S+)%s*%(([^)]*)%)") do
                    local param_list = {}
                    for param in params:gmatch("[^,]+") do
                        table.insert(param_list, param:match("^%s*(.-)%s*$"))
                    end
                    local parts = {}
                    for part in func_name:gmatch("[^_]+") do
                        table.insert(parts, part)
                    end
                    local package_name = table.concat(parts, "_", 2, #parts-2):gsub("_", ".")
                    local class_name = parts[#parts-1]
                    local method_name = parts[#parts]
                    local args = ""
                    for i, param in ipairs(param_list) do
                        if i == 1 then
                            args = format("_: %s,", param_map[param])
                        else
                            args = format("%s _: %s,", args, param_map[param])
                        end
                    end
                    local func = format([[#[jni_fn::jni_fn("%s.%s")]
pub fn %s(%s) %s{
    todo!();
}
]], package_name, class_name, method_name, args, (return_type == "void" and {""} or {format("-> %s ", param_map[return_type])})[1])
                    contents = format("%s%s\n", contents, func)
                end
            end
            print(format("Write to %s", output_file))
            io.writefile(output_file, contents)
        end
    end)
    set_menu {
        usage = "xmake generate_jni_funtions",
        description = "Generate jni funtions",
        options = {
        }
    }