#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(push_constant) uniform MatrixBlock {
	mat4 mv;
	mat4 mvp;
} Matrices;

layout(binding = 0) uniform ViewMatrixBlock {
	mat4 v;
} ViewMatrix;

layout(location = 0) out vec3 viewspace_pos;
layout(location = 1) out vec3 viewspace_normal;
layout(location = 2) out vec3 viewspace_lightvec;

vec3 worldspace_lightpos = vec3(0.0, 0.0, 0.0);

void main()
{
	// position is a point, set w to 1.0 and divide it out afterwards
	vec4 viewspace_pos4 = Matrices.mv * vec4(position, 1.0);
	viewspace_pos = vec3(viewspace_pos4) / viewspace_pos4.w;

	// normal is a vector, set w to 0.0
	viewspace_normal = vec3(Matrices.mv * vec4(normal, 0.0));

	// lightpos is a point, set w to 1.0 and divide it out afterwards
	vec4 viewspace_lightpos4 = ViewMatrix.v * vec4(worldspace_lightpos, 1.0);
	vec3 viewspace_lightpos = vec3(viewspace_lightpos4) / viewspace_lightpos4.w;
	viewspace_lightvec = viewspace_lightpos - viewspace_pos;

	gl_Position = Matrices.mvp * vec4(position, 1.0);
}
