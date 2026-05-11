// Swift launch bridge. SDL3 manages its own UIApplicationDelegate
// internally, so we just hand control over via SDL_RunApp.
//
// The Rust crate exports `SDL_main` (a C-ABI function) — SDL calls it
// after UIKit and the Metal surface are up.

import Foundation

@_cdecl("main")
func main(_ argc: Int32, _ argv: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?) -> Int32 {
    return SDL_RunApp(argc, argv, SDL_main, nil)
}

