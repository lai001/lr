namespace Foundation
{
    public class Locker<T>
    {
        public delegate void ActionRef(ref T? item);

        private T? value = default;
        private readonly object _object = new();

        public Locker()
        {

        }

        public Locker(T value)
        {
            this.value = value;
        }

        public void Set(T value)
        {
            lock (_object)
            {
                this.value = value;
            }
        }

        public T? Get()
        {
            lock (_object)
            {
                return value;
            }
        }

        public void Read(Action<T?> closure)
        {
            lock (_object)
            {
                closure(value);
            }
        }

        public void Write(ActionRef closure)
        {
            lock (_object)
            {
                closure(ref value);
            }
        }
    }
}
