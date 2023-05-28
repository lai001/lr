using Native;
using System;

namespace Script
{
    public interface IUserScript: IDisposable
    {
        string Name { get; }
        string Description { get; }

        public void Initialize();

        public void RedrawRequested(NativeTextureView nativeTextureView, NativeQueue nativeQueue);

        public void KeyboardInput(NativeKeyboardInput keyboardInput);

        public void CursorMoved(PhysicalPosition physicalPosition);
    }
}