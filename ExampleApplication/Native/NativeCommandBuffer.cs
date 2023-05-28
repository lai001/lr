using System.Runtime.InteropServices;

namespace Native
{
    using NativeCommandBufferType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeCommandBuffer
    {
        private bool isVaild = true;

        private NativeCommandBufferType nativeCommandBuffer = NativeCommandBufferType.Zero;

        public NativeCommandBuffer(NativeCommandBufferType nativeCommandBuffer)
        {
            this.nativeCommandBuffer = nativeCommandBuffer;
        }

        public NativeCommandBufferType GetNativeHandle()
        {
            if (!isVaild)
            {
                return NativeCommandBufferType.Zero;
            }
            return nativeCommandBuffer;
        }

        public void Submit(NativeQueue nativeQueue)
        {
            if (!isVaild)
            {
                throw new Exception("");
            }
            isVaild = false;
            NativeQueue.Functions.nativeQueueSubmit(nativeQueue.GetNativeHandle(), nativeCommandBuffer);
        }
    }
}
