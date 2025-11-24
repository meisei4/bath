CONCERNING:
cargo run --example raylib_test_1 --features tests-only
cargo run --example drekker_effect --features tests-only
cargo run --example feedback_buffer --features tests-only,glsl-330 (identical to raylib_test_1, but probably using trait wrapper)
cargo run --example ghost_dither_glsl100 --features tests-only,glsl-100


KIND OF FINE?
cargo run --example raylib_test_0 --features tests-only
cargo run --example ice_sheets --features tests-only
cargo run --example fft_visualizer --features tests-only
cargo run --example debug_space --features tests-only
cargo run --example rlgl_test --features tests-only
cargo run --example ghost_dither_cpu_shader --features tests-only
cargo run --example ghost_dither_opengl11_geometry --features tests-only,opengl-11
cargo run --example ghost_dither_opengl11_texture --features tests-only,opengl-11
cargo run --example core_3d_fixed_function_didactic --features tests-only,opengl-11
cargo run --example debug_mesh_info --features tests-only,opengl-11
cargo run --example ghost_dither_opengl11 --features tests-only,opengl-11
cargo run --example ghost_dither_opengl11_observation --features tests-only,opengl-11
cargo run --example music_ball --features tests-only
