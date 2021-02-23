# gcadapter-unity

This is a `libusb` driver written in Rust using [rusb](https://github.com/a1ien/rusb) for the [WUP-028 GameCube controller adapter](https://en.wikipedia.org/wiki/GameCube_controller#Super_Smash_Bros._Ultimate_Edition_controller).

### Features

 - Cross-platform (MacOS, Windows, Linux)
 - Low latency, multithreaded
 - Hotplug support

Example usage: [rust](https://github.com/veryjos/gcadapter-unity/blob/master/native/gcadapter_example/src/main.rs) / [c# bindings](https://github.com/veryjos/gcadapter-unity/blob/master/csharp/GamecubeAdapter.cs)
