// Minimal bridging header — exposes just enough of SDL3 to let Swift's
// @_cdecl("main") bridge call SDL_RunApp. We deliberately do NOT include
// <SDL3/SDL_main.h> because its preprocessor macros define `main` and
// confuse Swift's automatic entry-point handling.

#ifndef SAFIUI_BRIDGING_HEADER_H
#define SAFIUI_BRIDGING_HEADER_H

#include <stdint.h>

typedef int (*SDL_main_func)(int argc, char *argv[]);

extern int SDL_RunApp(int argc, char *argv[], SDL_main_func mainFunction, void *reserved);
extern int SDL_main(int argc, char *argv[]);

#endif /* SAFIUI_BRIDGING_HEADER_H */
