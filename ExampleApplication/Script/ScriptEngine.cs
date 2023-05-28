using Foundation;
using System;
using System.IO;
using System.Linq;
using HotReload;
using Native;

namespace Script
{
    public class UserSscript
    {
        private WeakReference userScriptWeakRef;

        public UserSscript(IUserScript userScript)
        {
            userScriptWeakRef = new WeakReference(userScript); 
        }

        public void Initialize()
        {
            if (userScriptWeakRef.Target != null)
            {
                IUserScript userScript = userScriptWeakRef.Target as IUserScript;
                userScript.Initialize();
            }
        }

        public void RedrawRequested(NativeTextureView nativeTextureView, NativeQueue nativeQueue)
        {
            if (userScriptWeakRef.Target != null)
            {
                IUserScript userScript = userScriptWeakRef.Target as IUserScript;
                userScript.RedrawRequested(nativeTextureView, nativeQueue);
            }
        }

        public void KeyboardInput(NativeKeyboardInput keyboardInput)
        {
            if (userScriptWeakRef.Target != null)
            {
                IUserScript userScript = userScriptWeakRef.Target as IUserScript;
                userScript.KeyboardInput(keyboardInput);
            }
        }

        public void CursorMoved(PhysicalPosition physicalPosition)
        {
            if (userScriptWeakRef.Target != null)
            {
                IUserScript userScript = userScriptWeakRef.Target as IUserScript;
                userScript.CursorMoved(physicalPosition);
            }
        }
    }

    public class ScriptEngine
    {
        public UserSscript  userSscript;

        public ScriptEngine()
        {
        }

        public void Reload()
        {
            string fileName = "UserScript.dll";
            RemoteAssembly.SetAssemblyPath($"./{fileName}");
            string source = Path.Join(Directory.GetCurrentDirectory(), "tmp", fileName);
            string target = Path.Join(Directory.GetCurrentDirectory(), fileName);
            try
            {
                RemoteAssembly.UnloadAssembly();
                File.Copy(source, target, true);
                Console.WriteLine($"copy {source} to {target}.");
                RemoteAssembly.LoadAssembly();
                userSscript = new UserSscript(Util.CreateInstances<IUserScript>(RemoteAssembly.GetAssembly()).First());
            }
            catch (Exception ex)
            {
                Console.WriteLine($"{ex}");
            }
        }
    }
}
