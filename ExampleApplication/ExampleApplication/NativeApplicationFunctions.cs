using System;
using Native;
using System.Runtime.InteropServices;
using Foundation;

namespace ExampleApplication
{
    using RuntimeApplicationType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeApplicationFunctions
    {
        private unsafe delegate* unmanaged<RuntimeApplicationType, NativeTextureView, NativeQueue, void> applicationRedrawRequested = &RedrawRequested;
        private unsafe delegate* unmanaged<RuntimeApplicationType, NativeKeyboardInput, void> applicationKeyboardInput = &KeyboardInput;
        private unsafe delegate* unmanaged<RuntimeApplicationType, PhysicalPosition, void> applicationCursorMoved = &CursorMoved;

        public NativeApplicationFunctions()
        {
        }

        [UnmanagedCallersOnly]
        private static unsafe void RedrawRequested(RuntimeApplicationType pointer, NativeTextureView nativeTextureView, NativeQueue nativeQueue)
        {
            Application application = UnmanagedObject<Application>.Cast(pointer);
            application.RedrawRequested(nativeTextureView, nativeQueue);
        }

        [UnmanagedCallersOnly]
        private static unsafe void KeyboardInput(RuntimeApplicationType pointer, NativeKeyboardInput keyboardInput)
        {
            Application application = UnmanagedObject<Application>.Cast(pointer);
            application.KeyboardInput(keyboardInput);
        }

        [UnmanagedCallersOnly]
        private static unsafe void CursorMoved(RuntimeApplicationType pointer, PhysicalPosition physicalPosition)
        {
            Application application = UnmanagedObject<Application>.Cast(pointer);
            application.CursorMoved(physicalPosition);
        }
    }
}
