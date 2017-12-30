#version 450
#extension GL_ARB_separate_shader_objects : enable

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) out vec2 texCoord;

vec2 positions[6] = vec2[](
    vec2(-1.0, -1.0), // T1: Bottom left
    vec2(1.0, -1.0),  // T1: Bottom right
    vec2(-1.0, 1.0),  // T1: Top left
    vec2(1.0, -1.0),  // T2: Bottom right
    vec2(1.0, 1.0),   // T2: Top right
    vec2(-1.0, 1.0)   // T2: Top left
);

vec2 tex_coords[6] = vec2[](
    vec2(0.0, 0.0), // T1: Bottom left
    vec2(1.0, 0.0), // T1: Bottom right
    vec2(0.0, 1.0), // T1: Top left
    vec2(1.0, 0.0), // T2: Bottom right
    vec2(1.0, 1.0), // T2: Top right
    vec2(0.0, 1.0)  // T2: Top left
);

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    texCoord = tex_coords[gl_VertexIndex];
}
