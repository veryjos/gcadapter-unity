using System;

namespace csharp
{
    class Program
    {
        static void Main(string[] args)
        {
            var adapter = new GamecubeAdapter();
            adapter.Start();

            while (true)
            {
                adapter.GetControllerStates();
                System.Threading.Thread.Sleep(50);
            }
        }
    }
}
