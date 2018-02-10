#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 texCoord;
layout(set = 0, location = 0) uniform sampler2D tex;
layout(location = 0) out vec4 outColor;

void main() {
    outColor = texture(tex, texCoord);
}
