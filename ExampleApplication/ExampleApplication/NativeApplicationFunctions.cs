using System;
using Native;
using System.Runtime.InteropServices;
using Foundation;

namespace ExampleApplication
{
    using RuntimeApplicationType = IntPtr;
    using NativeEngineType = IntPtr;

    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct NativeApplicationFunctions
    {
        private unsafe delegate* unmanaged<RuntimeApplicationType, NativeKeyboardInput, void> applicationKeyboardInput = &KeyboardInput;
        private unsafe delegate* unmanaged<RuntimeApplicationType, PhysicalPosition, void> applicationCursorMoved = &CursorMoved;
        private unsafe delegate* unmanaged<RuntimeApplicationType, NativeEngineType, void> applicationTick = &Tick;

        public NativeApplicationFunctions()
        {
        }

        [UnmanagedCallersOnly]
        private static unsafe void Tick(RuntimeApplicationType pointer, NativeEngineType nativeEngine)
        {
            Application application = UnmanagedObject<Application>.Cast(pointer);
            NativeEngine engine = new NativeEngine(nativeEngine);
            application.Tick(engine);
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
