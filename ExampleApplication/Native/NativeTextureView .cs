using System.Runtime.InteropServices;

namespace Native
{
    using NativeTextureViewType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeTextureView
    {
        private NativeTextureViewType nativeTextureView = NativeTextureViewType.Zero;

        public NativeTextureView(NativeTextureViewType nativeTextureView)
        {
            this.nativeTextureView = nativeTextureView;
        }

        public NativeTextureViewType GetNativeHandle()
        {
            return nativeTextureView;
        }
    }
}
