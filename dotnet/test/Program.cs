using System;

using GcAdapterLib;

namespace GcAdapterTest
{
    class Program
    {
        static void Main(string[] args)
        {
            // Create a new adapter daemon
            GcAdapter adapter = new GcAdapter(
                // Controller plugged in event
                (int controllerId) => {
                    Console.WriteLine("Controller plugged in: " + controllerId);
                },

                // Controller unplugged event
                (int controllerId) => {
                    Console.WriteLine("Controller unplugged: " + controllerId);
                },

                // Controller state update event
                (int controllerId) => {
                    Console.WriteLine("Controller state: " + controllerId);
                }
            );

            // Dispose the adapter daemon
            adapter.Dispose();
        }
    }
}
