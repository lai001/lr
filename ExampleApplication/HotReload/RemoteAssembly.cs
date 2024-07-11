using System.Reflection;
using System.Runtime.CompilerServices;

namespace HotReload
{
    public class RemoteAssembly
    {
        private static WeakReference? contextWeakRef;

        private static WeakReference? assemblyWeakRef;

        private static bool isLoaded = false;

        private static string? assemblyPath;

        public static void SetAssemblyPath(string inAssemblyPath)
        {
            assemblyPath = inAssemblyPath;
        }

        public static Assembly? GetAssembly()
        {
            if (assemblyWeakRef != null)
            {
                return assemblyWeakRef.Target as Assembly;
            }
            else
            {
                return null;
            }
        }

        [MethodImpl(MethodImplOptions.NoInlining)]
        public static void LoadAssembly()
        {
            if (isLoaded)
            {
                return;
            }
            if (assemblyPath != null)
            {
                var context = new HotReloadAssemblyLoadContext(assemblyPath);
                contextWeakRef = new WeakReference(context);
                //Assembly assembly = context.LoadFromAssemblyName(new AssemblyName(Path.GetFileNameWithoutExtension(assemblyPath)));
                Assembly assembly = context.LoadFromAssemblyPath(assemblyPath);
                assemblyWeakRef = new WeakReference(assembly);
                isLoaded = true;
            }
        }

        [MethodImpl(MethodImplOptions.NoInlining)]
        public static void UnloadAssembly()
        {
            if (!isLoaded)
            {
                return;
            }
            if (contextWeakRef != null && contextWeakRef.Target != null)
            {
                HotReloadAssemblyLoadContext? context = contextWeakRef.Target as HotReloadAssemblyLoadContext;
                if (context != null)
                {
                    context.Unload();
                    for (int i = 0; contextWeakRef.IsAlive && (i < 10); i++)
                    {
                        GC.Collect();
                        GC.WaitForPendingFinalizers();
                    }
                    isLoaded = false;
                }
            }
        }
    }
}
