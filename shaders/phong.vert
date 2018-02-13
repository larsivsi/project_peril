#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(push_constant) uniform MatrixBlock {
	mat4 mv;
	mat4 mvp;
} Matrices;
layout(location = 0) out vec3 viewspace_pos;
layout(location = 1) out vec3 viewspace_normal;

void main()
{
	// position is a point, set w to 1.0 and divide it out afterwards
	vec4 viewspace_pos4 = Matrices.mv * vec4(position, 1.0);
	viewspace_pos = vec3(viewspace_pos4) / viewspace_pos4.w;
	// normal is a vector, set w to 0.0
	viewspace_normal = vec3(Matrices.mv * vec4(normal, 0.0));

	gl_Position = Matrices.mvp * vec4(position, 1.0);
}
