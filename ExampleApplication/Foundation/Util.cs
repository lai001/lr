using System.Reflection;
using System.Runtime.InteropServices;

namespace Foundation
{
    public sealed class UnmanagedObject<T> where T : class
    {
        GCHandle? gCHandle = null;

        public UnmanagedObject(T @object)
        {
            gCHandle = GCHandle.Alloc(@object);
        }

        public IntPtr GetHandlePointer()
        {
            if (gCHandle != null)
            {
                return (IntPtr)gCHandle.Value;
            }
            else 
            { 
                return IntPtr.Zero;
            }
        }

        public void Free()
        {
            if (gCHandle != null)
            {
                gCHandle.Value.Free();
            }
        }

        public static T? Cast(IntPtr intPtr)
        {
            GCHandle gCHandle = GCHandle.FromIntPtr(intPtr);
            T? application = gCHandle.Target as T;
            return application;
        }
    }

    public sealed class CString : IDisposable
    {
        private IntPtr handle = IntPtr.Zero;

        private bool disposed;

        public CString(string str)
        {
            if (str != null)
            {
                handle = Marshal.StringToCoTaskMemUTF8(str);
            }
        }

        public IntPtr GetNativeHandle()
        {
            return handle;
        }

        void Dispose(bool disposing)
        {
            if (!disposed)
            {
                if (disposing)
                {
                    // Manual release of managed resources.
                }

                // Release unmanaged resources.
                Marshal.FreeCoTaskMem(handle);
                disposed = true;
            }
        }

        ~CString()
        {
            Dispose(disposing: false);
        }

        public void Dispose()
        {
            Dispose(disposing: true);
            GC.SuppressFinalize(this);
        }
    }

    public sealed class Util
    {
        public unsafe static IntPtr ToUnmanagedPtr<T>(T @object)
        {
            if (@object == null)
            {
                return IntPtr.Zero;
            }
            IntPtr intPtr = Marshal.AllocHGlobal(Marshal.SizeOf<T>());
            Marshal.StructureToPtr(@object, intPtr, false);
            return intPtr;
        }

        public static string GetAddress(IntPtr ptr)
        {
            return "0x" + ptr.ToString("X").ToLower();
        }

        public static IEnumerable<T> CreateInstances<T>(Assembly assembly) where T : class
        {
            int count = 0;

            foreach (Type type in assembly.GetTypes())
            {
                if (typeof(T).IsAssignableFrom(type))
                {
                    T? result = Activator.CreateInstance(type) as T;
                    if (result != null)
                    {
                        count++;
                        yield return result;
                    }
                }
            }

            if (count == 0)
            {
                string availableTypes = string.Join(",", assembly.GetTypes().Select(t => t.FullName));
                throw new ApplicationException(
                    $"Can't find any type which implements {typeof(T)} in {assembly} from {assembly.Location}.\n" +
                    $"Available types: {availableTypes}");
            }
        }
    }
}
