cd crates/renderer/renderer_cpp/shaders \
&& glslc object.vert -mfmt=c -o ../include/object_vert.h && glslc object.frag -mfmt=c -o ../include/object_frag.h \
&& glslc level.vert -mfmt=c -o ../include/level_vert.h && glslc level.frag -mfmt=c -o ../include/level_frag.h \
&& glslc ui.vert -mfmt=c -o ../include/ui_vert.h && glslc ui.frag -mfmt=c -o ../include/ui_frag.h \
&& glslc weapon.vert -mfmt=c -o ../include/weapon_vert.h && glslc weapon.frag -mfmt=c -o ../include/weapon_frag.h \
&& cd ../../../..
