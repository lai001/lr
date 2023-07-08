using System;
using System.Threading;
using System.Threading.Tasks;

namespace Foundation
{
    public static class ActionExtensions
    {
        public static Action Debounce(this Action func, int milliseconds = 300)
        {
            int last = 0;
            return delegate ()
            {
                int current = Interlocked.Increment(ref last);
                Task.Delay(milliseconds).ContinueWith(task =>
                {
                    if (current == last)
                    {
                        func();
                    }
                    task.Dispose();
                });
            };
        }
    }
}