using System.Runtime.InteropServices;

namespace Native
{
    using NativeQueueType = IntPtr;
    using NativeCommandBufferType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeQueueFunctions
    {
        public unsafe delegate* unmanaged<NativeQueueType, NativeCommandBufferType, void> nativeQueueSubmit;

    }

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeQueue
    {
        private NativeQueueType nativeQueue = NativeQueueType.Zero;

        public static NativeQueueFunctions Functions;

        public NativeQueue(NativeQueueType nativeQueue)
        {
            this.nativeQueue = nativeQueue;
        }

        public NativeQueueType GetNativeHandle()
        {
            return nativeQueue;
        }

        //public void Submit(NativeCommandBuffer nativeCommandBuffer)
        //{
        //    Functions.nativeQueueSubmit(nativeQueue, nativeCommandBuffer.GetNativeHandle());
        //}
    }
}
