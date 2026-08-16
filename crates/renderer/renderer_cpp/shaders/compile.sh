cd crates/renderer/renderer_cpp/shaders \
&& glslc sprite.vert -mfmt=c -o ../include/sprite_vert.h && glslc sprite.frag -mfmt=c -o ../include/sprite_frag.h \
&& glslc level.vert -mfmt=c -o ../include/level_vert.h && glslc level.frag -mfmt=c -o ../include/level_frag.h \
&& glslc ui.vert -mfmt=c -o ../include/ui_vert.h && glslc ui.frag -mfmt=c -o ../include/ui_frag.h \
&& cd ../../../..
