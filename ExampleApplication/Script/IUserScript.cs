using Native;
using System;

namespace Script
{
    public interface IUserScript : IDisposable
    {
        string Name { get; }
        string Description { get; }

        public void Initialize();

        public void Tick(NativeEngine engine);

        public void KeyboardInput(NativeKeyboardInput keyboardInput);

        public void CursorMoved(PhysicalPosition physicalPosition);
    }
}