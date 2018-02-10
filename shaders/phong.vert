#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(push_constant) uniform MatrixBlock
{
	mat4 v;
	mat4 mv;
	mat4 mvp;
} Matrices;
layout(location = 0) out vec3 v;
layout(location = 1) out vec3 N;

void main()
{
	vec4 v_4 = Matrices.mv * vec4(position, 1.0);
	v = vec3(v_4) / v_4.w;
	N = vec3(Matrices.mv * vec4(normal, 0.0));

	gl_Position = Matrices.mvp * vec4(position, 1.0);
}
