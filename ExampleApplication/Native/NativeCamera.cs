using System.Runtime.InteropServices;

namespace Native
{
    using NativeCameraType = IntPtr;
    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeCameraFunctions
    {
        public delegate* unmanaged<NativeCameraType, uint, uint, void> set_window_size;
        public delegate* unmanaged<NativeCameraType, void> drop;
    }

    public unsafe class NativeCamera : IDisposable
    {
        public static NativeCameraFunctions? Functions;

        public NativeCameraType nativeCamera = NativeCameraType.Zero;

        private bool disposed;

        public NativeCamera(NativeCameraType nativeCamera)
        {
            this.nativeCamera = nativeCamera;
        }

        ~NativeCamera()
        {
            Dispose(disposing: false);
        }

        public void Dispose()
        {
            Dispose(disposing: true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!disposed)
            {
                if (disposing)
                {
                    // Manual release of managed resources.
                }

                // Release unmanaged resources.
                System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
                Functions.Value.drop(nativeCamera);
                nativeCamera = NativeCameraType.Zero;
                disposed = true;
            }
        }

        public void SetWindowSize(uint windowWidth, uint windowHeight)
        {
            System.Diagnostics.Debug.Assert(Functions != null && Functions.HasValue);
            System.Diagnostics.Debug.Assert(nativeCamera != NativeCameraType.Zero);
            Functions.Value.set_window_size(nativeCamera, windowWidth, windowHeight);
        }
    }
}
